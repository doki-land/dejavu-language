//! The built-in filter registry is the cross-host single description.

use std::collections::HashSet;

use dejavu_ir::registry::{
    Capability, CapabilitySet, FilterLookupError, filter_spec, validate_filter,
};
use dejavu_ir::{
    IrDocument, IrError, RenderOptions, codes, default_language, render_ir, render_ir_opts,
};

#[test]
fn filter_names_and_aliases_are_unique() {
    let mut seen = HashSet::new();
    for spec in dejavu_ir::registry::FILTERS {
        assert!(seen.insert(spec.name), "duplicate filter name: {}", spec.name);
        for alias in spec.aliases {
            assert!(seen.insert(*alias), "duplicate filter alias: {alias}");
        }
    }
}

#[test]
fn aliases_resolve_to_the_canonical_spec() {
    assert_eq!(filter_spec("upper").unwrap().name, "uppercase");
    assert_eq!(filter_spec("e").unwrap().name, "escape");
    assert_eq!(filter_spec("raw").unwrap().name, "safe");
    assert!(filter_spec("nope").is_none());
}

#[test]
fn arity_is_enforced_per_filter() {
    assert!(validate_filter("replace", 2, CapabilitySet::all()).is_ok());
    assert!(matches!(
        validate_filter("replace", 1, CapabilitySet::all()),
        Err(FilterLookupError::Arity { spec }) if spec.name == "replace"
    ));
    assert!(matches!(
        validate_filter("default", 0, CapabilitySet::all()),
        Err(FilterLookupError::Arity { .. })
    ));
    assert!(matches!(
        validate_filter("missing", 0, CapabilitySet::all()),
        Err(FilterLookupError::Unknown)
    ));
}

#[test]
fn capability_allowlist_gates_output_owning_filters() {
    let pure_only = CapabilitySet::from_names(&["pure".to_string()]).expect("known capability");
    assert!(validate_filter("uppercase", 0, pure_only).is_ok());
    assert!(matches!(
        validate_filter("safe", 0, pure_only),
        Err(FilterLookupError::NotAllowed { spec }) if spec.capability == Capability::SafeOutput
    ));
    assert!(matches!(
        validate_filter("escape", 0, pure_only),
        Err(FilterLookupError::NotAllowed { .. })
    ));
    assert!(matches!(
        CapabilitySet::from_names(&["nope".to_string()]),
        Err(name) if name == "nope"
    ));
}

#[test]
fn render_options_capabilities_reject_safe_filters() {
    let doc = pipe_doc("safe", vec![]);
    let opts = RenderOptions { capabilities: CapabilitySet::from_names(&["pure".to_string()]).unwrap(), ..Default::default() };
    let err = render_ir_opts(&doc, &serde_json::json!({"x": "<b>"}), opts, None).expect_err("denied");
    assert_eq!(code(err), codes::CAPABILITY_NOT_ALLOWED);
}

#[test]
fn registry_json_lists_every_namespace_entry() {
    let json = dejavu_ir::registry::to_json();
    for spec in dejavu_ir::registry::FILTERS {
        assert!(json.contains(&format!("\"name\":\"{}\"", spec.name)), "{json}");
    }
    for spec in dejavu_ir::registry::FUNCTIONS {
        assert!(json.contains(&format!("\"name\":\"{}\"", spec.name)), "{json}");
    }
    for spec in dejavu_ir::registry::TESTS {
        assert!(json.contains(&format!("\"name\":\"{}\"", spec.name)), "{json}");
    }
    assert!(json.starts_with("{\"filters\":"), "{json}");
    assert!(json.contains("\"functions\":["), "{json}");
    assert!(json.contains("\"tests\":["), "{json}");
}

fn pipe_doc(filter: &str, args: Vec<serde_json::Value>) -> IrDocument {
    let doc = serde_json::json!({
        "irVersion": "v0",
        "language": default_language(),
        "body": { "type": "Template", "children": [{
            "type": "Interpolation",
            "trim": "none",
            "expression": {
                "type": "Expr.Pipe",
                "expression": { "type": "Expr.Literal", "value": "abc" },
                "filter": filter,
                "arguments": args,
            }
        }]}
    });
    serde_json::from_value(doc).expect("parse ir json")
}

fn code(err: IrError) -> String {
    match err {
        IrError::Diagnostic(diagnostic) => diagnostic.code,
        other => panic!("expected diagnostic, got {other:?}"),
    }
}

#[test]
fn renderer_rejects_out_of_contract_arguments() {
    let err = render_ir(&pipe_doc("replace", vec![]), &serde_json::json!({}))
        .expect_err("replace needs two arguments");
    assert_eq!(code(err), codes::FILTER_ARITY);
}

#[test]
fn renderer_rejects_unknown_filters() {
    let err = render_ir(&pipe_doc("nope", vec![]), &serde_json::json!({}))
        .expect_err("unknown filter");
    assert_eq!(code(err), codes::UNKNOWN_FILTER);
}
