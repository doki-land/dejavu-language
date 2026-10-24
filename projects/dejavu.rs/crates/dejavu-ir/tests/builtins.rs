//! Runtime semantics of the built-in `function` and `test` namespaces.

use dejavu_ir::registry::{
    Capability, CapabilitySet, FunctionLookupError, TestLookupError, function_spec, test_spec, validate_function,
    validate_test,
};
use dejavu_ir::{IrError, call_function, call_test, codes, default_language, render_ir};
use std::collections::HashSet;

#[test]
fn names_and_aliases_are_unique_per_namespace() {
    let mut seen = HashSet::new();
    for spec in dejavu_ir::registry::FUNCTIONS {
        assert!(seen.insert(spec.name), "duplicate function name: {}", spec.name);
        for alias in spec.aliases {
            assert!(seen.insert(*alias), "duplicate function alias: {alias}");
        }
    }
    let mut seen = HashSet::new();
    for spec in dejavu_ir::registry::TESTS {
        assert!(seen.insert(spec.name), "duplicate test name: {}", spec.name);
        for alias in spec.aliases {
            assert!(seen.insert(*alias), "duplicate test alias: {alias}");
        }
    }
}

#[test]
fn aliases_resolve_to_canonical_specs() {
    assert_eq!(function_spec("range").unwrap().name, "range");
    assert_eq!(test_spec("none").unwrap().name, "null");
    assert!(function_spec("nope").is_none());
    assert!(test_spec("nope").is_none());
}

#[test]
fn arity_is_enforced() {
    assert!(validate_function("range", 2, CapabilitySet::all()).is_ok());
    assert!(validate_function("range", 3, CapabilitySet::all()).is_ok());
    assert!(matches!(
        validate_function("range", 1, CapabilitySet::all()),
        Err(FunctionLookupError::Arity { spec }) if spec.name == "range"
    ));
    assert!(matches!(validate_test("empty", 0, CapabilitySet::all()), Ok(_)));
    assert!(matches!(validate_test("empty", 1, CapabilitySet::all()), Err(TestLookupError::Arity { .. })));
    assert!(matches!(validate_test("nope", 0, CapabilitySet::all()), Err(TestLookupError::Unknown)));
}

#[test]
fn range_produces_integer_sequences() {
    let args = [serde_json::json!(0), serde_json::json!(3)];
    assert_eq!(call_function("range", &args, CapabilitySet::all()).unwrap(), serde_json::json!([0, 1, 2]));

    let args = [serde_json::json!(3), serde_json::json!(0), serde_json::json!(-1)];
    assert_eq!(call_function("range", &args, CapabilitySet::all()).unwrap(), serde_json::json!([3, 2, 1]));
}

#[test]
fn range_rejects_invalid_arguments_with_null() {
    let zero_step = [serde_json::json!(0), serde_json::json!(3), serde_json::json!(0)];
    assert_eq!(call_function("range", &zero_step, CapabilitySet::all()).unwrap(), serde_json::Value::Null);

    let fractional = [serde_json::json!(0.5), serde_json::json!(3)];
    assert_eq!(call_function("range", &fractional, CapabilitySet::all()).unwrap(), serde_json::Value::Null);

    let runaway = [serde_json::json!(0), serde_json::json!(1_000_000)];
    assert_eq!(call_function("range", &runaway, CapabilitySet::all()).unwrap(), serde_json::Value::Null);
}

#[test]
fn min_and_max_pick_finite_numbers() {
    let args = [serde_json::json!(4), serde_json::json!(2.5)];
    assert_eq!(call_function("min", &args, CapabilitySet::all()).unwrap(), serde_json::json!(2.5));
    assert_eq!(call_function("max", &args, CapabilitySet::all()).unwrap(), serde_json::json!(4));

    let mixed = [serde_json::json!(4), serde_json::json!("x")];
    assert_eq!(call_function("min", &mixed, CapabilitySet::all()).unwrap(), serde_json::Value::Null);
}

#[test]
fn tests_cover_value_model_predicates() {
    let empty: serde_json::Value = serde_json::json!([]);
    assert!(call_test("empty", &empty, &[], false, CapabilitySet::all()).unwrap());
    assert!(!call_test("empty", &empty, &[], true, CapabilitySet::all()).unwrap());
    assert!(call_test("array", &empty, &[], false, CapabilitySet::all()).unwrap());
    assert!(call_test("truthy", &serde_json::json!("0"), &[], false, CapabilitySet::all()).unwrap());
    assert!(!call_test("truthy", &serde_json::json!(0), &[], false, CapabilitySet::all()).unwrap());
    assert!(call_test("null", &serde_json::Value::Null, &[], false, CapabilitySet::all()).unwrap());
    assert!(call_test("even", &serde_json::json!(4), &[], false, CapabilitySet::all()).unwrap());
    assert!(!call_test("even", &serde_json::json!(4.5), &[], false, CapabilitySet::all()).unwrap());
    assert!(call_test("odd", &serde_json::json!(-3), &[], false, CapabilitySet::all()).unwrap());
    assert!(call_test("object", &serde_json::json!({"a": 1}), &[], false, CapabilitySet::all()).unwrap());
    assert!(call_test("string", &serde_json::json!("x"), &[], false, CapabilitySet::all()).unwrap());
    assert!(call_test("number", &serde_json::json!(1), &[], false, CapabilitySet::all()).unwrap());
    assert!(call_test("boolean", &serde_json::json!(true), &[], false, CapabilitySet::all()).unwrap());
}

#[test]
fn renderer_reports_unknown_function_and_test() {
    let json = serde_json::json!({
        "irVersion": "1.0",
        "language": default_language(),
        "body": { "type": "Template", "children": [{
            "type": "Interpolation",
            "trim": "none",
            "expression": {
                "type": "Expr.Call",
                "callee": { "type": "Expr.Identifier", "name": "nope" },
                "arguments": []
            }
        }]}
    });
    let doc = serde_json::from_value(json).expect("parse ir json");
    assert_eq!(error_code(render_ir(&doc, &serde_json::json!({})).expect_err("unknown function")), codes::UNKNOWN_FUNCTION);

    let json = serde_json::json!({
        "irVersion": "1.0",
        "language": default_language(),
        "body": { "type": "Template", "children": [{
            "type": "Interpolation",
            "trim": "none",
            "expression": {
                "type": "Expr.Test",
                "expression": { "type": "Expr.Identifier", "name": "x" },
                "test": "nope"
            }
        }]}
    });
    let doc = serde_json::from_value(json).expect("parse ir json");
    assert_eq!(error_code(render_ir(&doc, &serde_json::json!({"x": 1})).expect_err("unknown test")), codes::UNKNOWN_TEST);
}

#[test]
fn renderer_rejects_member_callee() {
    let json = serde_json::json!({
        "irVersion": "1.0",
        "language": default_language(),
        "body": { "type": "Template", "children": [{
            "type": "Interpolation",
            "trim": "none",
            "expression": {
                "type": "Expr.Call",
                "callee": {
                    "type": "Expr.Member",
                    "object": { "type": "Expr.Identifier", "name": "user" },
                    "property": "format"
                },
                "arguments": []
            }
        }]}
    });
    let doc = serde_json::from_value(json).expect("parse ir json");
    assert_eq!(
        error_code(
            render_ir(&doc, &serde_json::json!({"user": {"format": "x"}})).expect_err("member callee")
        ),
        codes::CALLS_NOT_SUPPORTED
    );
}

#[test]
fn renderer_evaluates_functions_and_tests_end_to_end() {
    let json = serde_json::json!({
        "irVersion": "1.0",
        "language": default_language(),
        "body": { "type": "Template", "children": [{
            "type": "Interpolation",
            "trim": "none",
            "expression": {
                "type": "Expr.Test",
                "expression": {
                    "type": "Expr.Call",
                    "callee": { "type": "Expr.Identifier", "name": "range" },
                    "arguments": [
                        { "type": "Expr.Literal", "value": 0 },
                        { "type": "Expr.Literal", "value": 4 }
                    ]
                },
                "test": "array"
            }
        }]}
    });
    let doc = serde_json::from_value(json).expect("parse ir json");
    assert_eq!(render_ir(&doc, &serde_json::json!({})).unwrap(), "true");
}

#[test]
fn capability_allowlist_gates_functions_and_tests() {
    // Functions and tests need the `pure` capability the host grants.
    assert!(validate_function("min", 2, CapabilitySet::all()).is_ok());
    assert!(matches!(
        validate_function("min", 2, CapabilitySet::none()),
        Err(FunctionLookupError::NotAllowed { spec }) if spec.capability == Capability::Pure
    ));
    assert!(matches!(
        validate_test("empty", 0, CapabilitySet::none()),
        Err(TestLookupError::NotAllowed { spec }) if spec.capability == Capability::Pure
    ));

    let err = call_function("min", &[serde_json::json!(1), serde_json::json!(2)], CapabilitySet::none())
        .expect_err("denied function");
    assert_eq!(error_code(err), codes::CAPABILITY_NOT_ALLOWED);

    let err = call_test("empty", &serde_json::json!([1]), &[], false, CapabilitySet::none())
        .expect_err("denied test");
    assert_eq!(error_code(err), codes::CAPABILITY_NOT_ALLOWED);
}

fn error_code(err: IrError) -> String {
    match err {
        IrError::Diagnostic(diagnostic) => diagnostic.code,
        other => panic!("expected diagnostic, got {other:?}"),
    }
}
