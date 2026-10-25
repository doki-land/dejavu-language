//! Host-supplied registry descriptions: wire shape, conflict rejection and the
//! merged cross-host dump.

use dejavu_ir::registry::{CustomEntry, Namespace, validate_custom_entries};
use dejavu_ir::{
    ArgValue, Capability, CustomError, CustomRegistry, IrDocument, IrError, RenderOptions, Stage, codes,
    default_language, render_ir_opts,
};
use serde_json::{Value, json};

/// A host that implements every entry it describes.
struct Host {
    entries: Vec<CustomEntry>,
    /// When set, every invocation fails with this message.
    fail: Option<String>,
}

impl Host {
    fn new(entries: Vec<CustomEntry>) -> Self {
        Self { entries, fail: None }
    }

    fn failing(entries: Vec<CustomEntry>, message: &str) -> Self {
        Self { entries, fail: Some(message.to_string()) }
    }

    fn outcome<T>(&self, value: T) -> Result<T, CustomError> {
        match &self.fail {
            Some(message) => Err(CustomError::new(message.clone())),
            None => Ok(value),
        }
    }
}

impl CustomRegistry for Host {
    fn entries(&self) -> &[CustomEntry] {
        &self.entries
    }

    fn apply_filter(&self, name: &str, value: &Value, _args: &[Value]) -> Result<Value, CustomError> {
        let text = value.as_str().unwrap_or_default();
        self.outcome(match name {
            "shout" => Value::String(format!("{text}!")),
            "bolder" => Value::String(format!("<b>{text}</b>")),
            other => panic!("unexpected filter dispatch: {other}"),
        })
    }

    fn call_function(&self, name: &str, args: &[Value]) -> Result<Value, CustomError> {
        self.outcome(match name {
            "twice" => json!(args[0].as_i64().unwrap_or_default() * 2),
            other => panic!("unexpected function dispatch: {other}"),
        })
    }

    fn apply_test(&self, name: &str, value: &Value, _args: &[Value]) -> Result<bool, CustomError> {
        self.outcome(match name {
            "evenish" => value.as_i64().is_some_and(|n| n % 2 == 0),
            other => panic!("unexpected test dispatch: {other}"),
        })
    }
}

fn literals(args: Vec<Value>) -> Vec<Value> {
    args.into_iter().map(|value| json!({ "type": "Expr.Literal", "value": value })).collect()
}

fn pipe_doc(filter: &str, value: &str, args: Vec<Value>) -> IrDocument {
    let doc = json!({
        "irVersion": "1.0",
        "language": default_language(),
        "body": { "type": "Template", "children": [{
            "type": "Interpolation",
            "trim": "none",
            "expression": {
                "type": "Expr.Pipe",
                "expression": { "type": "Expr.Literal", "value": value },
                "filter": filter,
                "arguments": literals(args),
            }
        }]}
    });
    serde_json::from_value(doc).expect("parse ir json")
}

fn call_doc(name: &str, args: Vec<Value>) -> IrDocument {
    let doc = json!({
        "irVersion": "1.0",
        "language": default_language(),
        "body": { "type": "Template", "children": [{
            "type": "Interpolation",
            "trim": "none",
            "expression": {
                "type": "Expr.Call",
                "callee": { "type": "Expr.Identifier", "name": name },
                "arguments": literals(args),
            }
        }]}
    });
    serde_json::from_value(doc).expect("parse ir json")
}

fn test_doc(name: &str, negated: bool) -> IrDocument {
    let doc = json!({
        "irVersion": "1.0",
        "language": default_language(),
        "body": { "type": "Template", "children": [{
            "type": "Stmt.If",
            "test": {
                "type": "Expr.Test",
                "expression": { "type": "Expr.Literal", "value": 4 },
                "test": name,
                "negated": negated,
            },
            "consequent": [{ "type": "Text", "value": "yes", "trim": "none" }],
            "elseIfs": [],
            "alternate": [],
            "trim": "none"
        }]}
    });
    serde_json::from_value(doc).expect("parse ir json")
}

fn host() -> Host {
    Host::new(vec![
        entry(Namespace::Filter, "shout", &["loud"], 0, 0, Capability::Pure),
        entry(Namespace::Filter, "bolder", &[], 0, 0, Capability::SafeOutput),
        entry(Namespace::Function, "twice", &[], 1, 1, Capability::Pure),
        entry(Namespace::Test, "evenish", &[], 0, 0, Capability::Pure),
    ])
}

fn render(doc: &IrDocument, custom: &dyn CustomRegistry, capabilities: dejavu_ir::CapabilitySet) -> Result<String, IrError> {
    render_ir_opts(
        doc,
        &json!({}),
        RenderOptions { capabilities, custom: Some(custom), ..Default::default() },
        None,
    )
}

fn code(err: IrError) -> String {
    match err {
        IrError::Diagnostic(diagnostic) => diagnostic.code,
        other => panic!("expected diagnostic, got {other:?}"),
    }
}

#[test]
fn custom_filter_dispatches_by_canonical_name_for_aliases() {
    let host = host();
    let doc = pipe_doc("loud", "hi", vec![]);
    assert_eq!(render(&doc, &host, dejavu_ir::CapabilitySet::all()).unwrap(), "hi!");
}

#[test]
fn custom_safe_output_filter_skips_escaping() {
    let host = host();
    let doc = pipe_doc("bolder", "hi", vec![]);
    assert_eq!(render(&doc, &host, dejavu_ir::CapabilitySet::all()).unwrap(), "<b>hi</b>");

    // Without the capability grant the same render is rejected.
    let pure = dejavu_ir::CapabilitySet::from_names(&["pure".to_string()]).unwrap();
    let err = render(&doc, &host, pure).expect_err("safe-output not granted");
    assert_eq!(code(err), codes::CAPABILITY_NOT_ALLOWED);
}

#[test]
fn custom_function_and_test_run_end_to_end() {
    let host = host();
    let doc = call_doc("twice", vec![json!(21)]);
    assert_eq!(render(&doc, &host, dejavu_ir::CapabilitySet::all()).unwrap(), "42");

    let doc = test_doc("evenish", false);
    assert_eq!(render(&doc, &host, dejavu_ir::CapabilitySet::all()).unwrap(), "yes");
    let doc = test_doc("evenish", true);
    assert_eq!(render(&doc, &host, dejavu_ir::CapabilitySet::all()).unwrap(), "");
}

#[test]
fn custom_arity_and_unknown_names_keep_builtin_codes() {
    let host = Host::new(vec![entry(Namespace::Function, "twice", &[], 1, 1, Capability::Pure)]);
    let doc = call_doc("twice", vec![]);
    assert_eq!(code(render(&doc, &host, dejavu_ir::CapabilitySet::all()).expect_err("arity")), codes::FUNCTION_ARITY);

    let doc = call_doc("nope", vec![]);
    assert_eq!(code(render(&doc, &host, dejavu_ir::CapabilitySet::all()).expect_err("unknown")), codes::UNKNOWN_FUNCTION);
}

#[test]
fn host_failures_surface_as_custom_invocation_for_every_namespace() {
    let entries = vec![
        entry(Namespace::Filter, "shout", &[], 0, 0, Capability::Pure),
        entry(Namespace::Function, "twice", &[], 1, 1, Capability::Pure),
        entry(Namespace::Test, "evenish", &[], 0, 0, Capability::Pure),
    ];
    let host = Host::failing(entries, "host exploded");

    let cases: [(&str, IrDocument); 3] = [
        ("filter", pipe_doc("shout", "hi", vec![])),
        ("function", call_doc("twice", vec![json!(21)])),
        ("test", test_doc("evenish", false)),
    ];
    for (namespace, doc) in cases {
        let err = render(&doc, &host, dejavu_ir::CapabilitySet::all()).expect_err("host failure");
        let IrError::Diagnostic(diagnostic) = err else { panic!("expected diagnostic") };
        assert_eq!(diagnostic.code, codes::CUSTOM_INVOCATION, "{namespace}");
        assert_eq!(arg_string(&diagnostic, "namespace"), namespace);
        assert_eq!(arg_string(&diagnostic, "message"), "host exploded");
    }
}

#[test]
fn conflicting_registration_is_rejected_before_rendering() {
    let host = Host::new(vec![entry(Namespace::Filter, "uppercase", &[], 0, 0, Capability::Pure)]);
    let doc = pipe_doc("uppercase", "x", vec![]);
    let err = render(&doc, &host, dejavu_ir::CapabilitySet::all()).expect_err("conflict");
    let IrError::Diagnostic(diagnostic) = err else { panic!("expected diagnostic") };
    assert_eq!(diagnostic.code, codes::REGISTRY_CONFLICT);
    assert_eq!(diagnostic.stage, Stage::Host);
    assert_eq!(arg_string(&diagnostic, "namespace"), "filter");
}


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
