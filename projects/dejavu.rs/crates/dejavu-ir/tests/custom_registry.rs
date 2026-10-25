//! Host-supplied registry descriptions: wire shape, conflict rejection and the
//! merged cross-host dump.

use dejavu_ir::registry::{CustomEntry, Namespace, validate_custom_entries};
use dejavu_ir::{ArgValue, Capability, Stage, codes};

fn arg_string(diagnostic: &dejavu_ir::Diagnostic, key: &str) -> String {
    match diagnostic.args.get(key) {
        Some(ArgValue::String(value)) => value.clone(),
        other => panic!("expected string arg `{key}`, got {other:?}"),
    }
}

fn entry(
    namespace: Namespace,
    name: &str,
    aliases: &[&str],
    min_args: usize,
    max_args: usize,
    capability: Capability,
) -> CustomEntry {
    CustomEntry {
        namespace,
        name: name.to_string(),
        aliases: aliases.iter().map(|alias| alias.to_string()).collect(),
        min_args,
        max_args,
        capability,
    }
}

fn filter_entry(name: &str) -> CustomEntry {
    entry(Namespace::Filter, name, &[], 0, 0, Capability::Pure)
}

#[test]
fn wire_shape_round_trips_with_camel_case_and_kebab_capability() {
    let json = r#"{"namespace":"safe-output","name":"x"}"#;
    let err = serde_json::from_str::<CustomEntry>(json);
    assert!(err.is_err(), "unknown namespace must not deserialize");

    let wire = r#"{
        "namespace": "filter",
        "name": "shout",
        "aliases": ["loud"],
        "minArgs": 1,
        "maxArgs": 2,
        "capability": "safe-output"
    }"#;
    let parsed: CustomEntry = serde_json::from_str(wire).expect("valid descriptor");
    assert_eq!(parsed.namespace, Namespace::Filter);
    assert_eq!(parsed.name, "shout");
    assert_eq!(parsed.aliases, vec!["loud".to_string()]);
    assert_eq!(parsed.min_args, 1);
    assert_eq!(parsed.max_args, 2);
    assert_eq!(parsed.capability, Capability::SafeOutput);

    let back = serde_json::to_value(&parsed).expect("serializes");
    assert_eq!(back["namespace"], "filter");
    assert_eq!(back["minArgs"], 1);
    assert_eq!(back["capability"], "safe-output");
    assert!(back.get("min_args").is_none());
}

#[test]
fn canonical_name_colliding_with_a_builtin_is_rejected() {
    let error = validate_custom_entries(&[filter_entry("uppercase")]).expect_err("conflict");
    assert_eq!(error.code, codes::REGISTRY_CONFLICT);
    assert_eq!(error.stage, Stage::Host);
    assert_eq!(arg_string(&error, "namespace"), "filter");
    assert_eq!(arg_string(&error, "name"), "uppercase");
}

#[test]
fn alias_colliding_with_a_builtin_alias_is_rejected() {
    let custom = entry(Namespace::Filter, "shout", &["upper"], 0, 0, Capability::Pure);
    let error = validate_custom_entries(&[custom]).expect_err("conflict");
    assert_eq!(error.code, codes::REGISTRY_CONFLICT);
    assert_eq!(arg_string(&error, "name"), "upper");
}

#[test]
fn two_custom_entries_sharing_a_spelling_are_rejected() {
    let first = entry(Namespace::Filter, "shout", &["loud"], 0, 0, Capability::Pure);
    let second = entry(Namespace::Function, "loud", &[], 0, 0, Capability::Pure);
    // Different namespaces may reuse a spelling.
    validate_custom_entries(&[first, second]).expect("namespaces are independent");

    let first = entry(Namespace::Filter, "shout", &["loud"], 0, 0, Capability::Pure);
    let second = entry(Namespace::Filter, "loud", &[], 0, 0, Capability::Pure);
    let error = validate_custom_entries(&[first, second]).expect_err("conflict");
    assert_eq!(error.code, codes::REGISTRY_CONFLICT);
    assert_eq!(arg_string(&error, "name"), "loud");
}

#[test]
fn distinctive_entries_are_accepted_across_namespaces() {
    let entries = vec![
        entry(Namespace::Filter, "shout", &["loud"], 1, 1, Capability::Pure),
        entry(Namespace::Function, "twice", &[], 1, 1, Capability::Pure),
        entry(Namespace::Test, "evenish", &[], 0, 0, Capability::Pure),
    ];
    validate_custom_entries(&entries).expect("no conflicts");
}

#[test]
fn merged_dump_appends_custom_entries_in_input_order() {
    let custom = vec![
        entry(Namespace::Filter, "shout", &["loud"], 1, 1, Capability::Pure),
        entry(Namespace::Test, "evenish", &[], 0, 0, Capability::Pure),
    ];
    let merged = dejavu_ir::registry::to_json_with_custom(&custom);
    let value: serde_json::Value = serde_json::from_str(&merged).expect("dump is valid json");

    let filters = value["filters"].as_array().expect("filters array");
    let last = filters.last().expect("custom filter appended");
    assert_eq!(last["name"], "shout");
    assert_eq!(last["aliases"], serde_json::json!(["loud"]));
    assert_eq!(last["minArgs"], 1);
    assert_eq!(last["capability"], "pure");

    let tests = value["tests"].as_array().expect("tests array");
    assert_eq!(tests.last().expect("custom test appended")["name"], "evenish");

    // Built-ins stay first and unchanged.
    assert_eq!(filters.first().expect("builtin first")["name"], "uppercase");
    let plain: serde_json::Value = serde_json::from_str(&dejavu_ir::registry::to_json()).expect("plain dump");
    assert_eq!(plain["filters"].as_array().expect("builtins").len(), filters.len() - 1);
}
