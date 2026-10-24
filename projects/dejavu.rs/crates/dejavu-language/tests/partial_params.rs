//! Parameterized `partial` declarations at the parser/IR boundary.

use dejavu_ir::types::{IrDocument, IrNode};
use dejavu_language::parse;

fn partial_node(source: &str) -> IrNode {
    let doc: IrDocument = parse(source).expect("parse");
    let IrNode::Template { children, .. } = doc.body else {
        panic!("expected Template body");
    };
    children
        .into_iter()
        .find(|c| matches!(c, IrNode::StmtPartial { .. }))
        .expect("Stmt.Partial")
}

#[test]
fn partial_declares_named_parameters_with_literal_defaults() {
    let IrNode::StmtPartial { params, body, .. } =
        partial_node(r#"<% partial title, suffix = "!" %>Hi<% end partial %>"#)
    else {
        unreachable!()
    };
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name, "title");
    assert!(params[0].default.is_none());
    assert_eq!(params[1].name, "suffix");
    assert!(matches!(
        params[1].default.as_ref(),
        Some(IrNode::ExprLiteral { value, .. }) if *value == serde_json::json!("!")
    ));
    assert_eq!(body.len(), 1);
}

#[test]
fn partial_requires_at_least_one_parameter() {
    let err = parse("<% partial %><% end partial %>").expect_err("empty parameter list");
    assert!(err.message.contains("at least one parameter"), "message: {}", err.message);
}

#[test]
fn partial_rejects_duplicate_parameters() {
    let err = parse(r#"<% partial a, a = 1 %>x<% end partial %>"#).expect_err("duplicate");
    assert!(err.message.contains("duplicate partial parameter"), "message: {}", err.message);
}

#[test]
fn partial_rejects_non_literal_defaults() {
    let err = parse("<% partial a = b %>x<% end partial %>").expect_err("non-literal default");
    assert!(err.message.contains("must be a literal"), "message: {}", err.message);
}

#[test]
fn partial_requires_end_partial() {
    let err = parse("<% partial a %>x").expect_err("unclosed partial");
    assert!(err.message.contains("expected `end partial`") || err.message.contains("code open"),
        "message: {}", err.message);
}

#[test]
fn partial_must_span_the_whole_template() {
    let err = parse("<% partial a %>x<% end partial %>tail").expect_err("trailing content");
    assert!(err.message.contains("whole template"), "message: {}", err.message);
}

#[test]
fn partial_cannot_be_nested() {
    let err = parse("<% if ok %><% partial a %>x<% end partial %><% end if %>")
        .expect_err("nested partial");
    assert!(err.message.contains("cannot be nested"), "message: {}", err.message);
}

#[test]
fn partial_ir_round_trips_through_serde() {
    let doc = parse(r#"<% partial title, suffix = "!" %><% title %><% end partial %>"#).expect("parse");
    let json = serde_json::to_string(&doc).expect("serialize");
    let back: IrDocument = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(doc, back);
}
