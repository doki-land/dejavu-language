//! Object literal expressions: parsing, key uniqueness, and IR round-trip.

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
fn object_literal_parses_keys_and_values() {
    let expr = interpolation_expr(r#"<% {name: "x", "count": total} %>"#);
    let IrNode::ExprObject { entries, .. } = expr else {
        panic!("expected Expr.Object");
    };
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].key, "name");
    assert!(matches!(&entries[0].value, IrNode::ExprLiteral { value, .. } if value == "x"));
    assert_eq!(entries[1].key, "count");
    assert!(matches!(&entries[1].value, IrNode::ExprIdentifier { name, .. } if name == "total"));
}

#[test]
fn nested_object_literal_and_trailing_comma() {
    let expr = interpolation_expr(r#"<% {outer: {inner: 1,},} %>"#);
    let IrNode::ExprObject { entries, .. } = expr else {
        panic!("expected Expr.Object");
    };
    assert!(matches!(&entries[0].value, IrNode::ExprObject { entries, .. } if entries.len() == 1));
}

#[test]
fn duplicate_object_key_is_rejected() {
    let err = parse(r#"<% {a: 1, a: 2} %>"#).expect_err("duplicate key must fail");
    assert!(err.message.contains("duplicate object key `a`"), "message: {}", err.message);
}

#[test]
fn object_literal_is_usable_as_include_context() {
    let doc: IrDocument = parse(r#"<% include "card" with {title: post.title, extra: 1} only %>"#)
        .expect("parse");
    let IrNode::Template { children, .. } = doc.body else {
        panic!("expected Template body");
    };
    let include = children
        .into_iter()
        .find(|c| matches!(c, IrNode::StmtInclude { .. }))
        .expect("include");
    let IrNode::StmtInclude { with, only, .. } = include else {
        unreachable!()
    };
    assert!(only);
    assert!(matches!(with.as_deref(), Some(IrNode::ExprObject { entries, .. }) if entries.len() == 2));
}

#[test]
fn object_literal_round_trips_through_serde() {
    let doc: IrDocument = parse(r#"<% {a: 1, b: "x"} %>"#).expect("parse");
    let json = serde_json::to_string(&doc).expect("serialize");
    let back: IrDocument = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back, doc);
}
