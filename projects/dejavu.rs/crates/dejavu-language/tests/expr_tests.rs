//! `is [not] <test>` expressions and closed `name(args)` calls.

use dejavu_ir::types::{IrDocument, IrNode};
use dejavu_language::parse;

fn interpolation_expr(source: &str) -> IrNode {
    let doc: IrDocument = parse(source).expect("parse");
    let IrNode::Template { children, .. } = doc.body else {
        panic!("expected Template body");
    };
    children
        .into_iter()
        .find_map(|c| match c {
            IrNode::Interpolation { expression, .. } => Some(*expression),
            _ => None,
        })
        .expect("interpolation")
}

#[test]
fn plain_test_is_encoded() {
    let expr = interpolation_expr("<% value is empty %>");
    let IrNode::ExprTest { expression, test, arguments, negated, .. } = expr else {
        panic!("expected Expr.Test");
    };
    assert_eq!(test, "empty");
    assert!(arguments.is_empty());
    assert!(!negated);
    assert!(matches!(*expression, IrNode::ExprIdentifier { name, .. } if name == "value"));
}

#[test]
fn negated_test_and_null_keyword() {
    let expr = interpolation_expr("<% user is not null %>");
    let IrNode::ExprTest { expression, test, negated, .. } = expr else {
        panic!("expected Expr.Test");
    };
    assert_eq!(test, "null");
    assert!(negated);
    assert!(matches!(*expression, IrNode::ExprIdentifier { name, .. } if name == "user"));
}

#[test]
fn test_arguments_are_parsed() {
    let expr = interpolation_expr("<% total is divisibleby(3) %>");
    let IrNode::ExprTest { test, arguments, .. } = expr else {
        panic!("expected Expr.Test");
    };
    assert_eq!(test, "divisibleby");
    assert_eq!(arguments.len(), 1);
    assert!(matches!(&arguments[0], IrNode::ExprLiteral { value, .. } if value == 3));
}

#[test]
fn test_binds_tighter_than_and() {
    let expr = interpolation_expr("<% a is string && b is number %>");
    let IrNode::ExprBinary { operator, left, right, .. } = expr else {
        panic!("expected Expr.Binary");
    };
    assert_eq!(operator, "&&");
    assert!(matches!(*left, IrNode::ExprTest { .. }));
    assert!(matches!(*right, IrNode::ExprTest { .. }));
}

#[test]
fn test_round_trips_through_serde() {
    let doc: IrDocument = parse("<% value is not empty %>").expect("parse");
    let json = serde_json::to_string(&doc).expect("serialize");
    let back: IrDocument = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, doc);
}

#[test]
fn missing_test_name_is_rejected() {
    let err = parse("<% value is %>").expect_err("must fail");
    assert!(err.message.contains("expected test name"), "message: {}", err.message);
}

#[test]
fn call_expression_is_parsed() {
    let expr = interpolation_expr("<% range(0, 3) %>");
    let IrNode::ExprCall { callee, arguments, .. } = expr else {
        panic!("expected Expr.Call");
    };
    assert!(matches!(*callee, IrNode::ExprIdentifier { name, .. } if name == "range"));
    assert_eq!(arguments.len(), 2);
}
