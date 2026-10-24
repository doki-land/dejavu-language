//! `macro` declarations at the parser/IR boundary.

use dejavu_ir::types::{IrDocument, IrNode};
use dejavu_language::parse;

fn macro_node(source: &str) -> IrNode {
    let doc: IrDocument = parse(source).expect("parse");
    let IrNode::Template { children, .. } = doc.body else {
        panic!("expected Template body");
    };
    children
        .into_iter()
        .find(|c| matches!(c, IrNode::StmtMacro { .. }))
        .expect("Stmt.Macro")
}

#[test]
fn macro_declares_name_and_literal_defaults() {
    let IrNode::StmtMacro { name, params, body, .. } =
        macro_node(r#"<% macro card(title, suffix = "!") %><% title %><% end macro %>"#)
    else {
        unreachable!()
    };
    assert_eq!(name, "card");
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
fn macro_accepts_an_empty_parameter_list() {
    let IrNode::StmtMacro { params, .. } = macro_node("<% macro divider() %>---<% end macro %>") else {
        unreachable!()
    };
    assert!(params.is_empty());
}

#[test]
fn macro_requires_parenthesised_parameters() {
    let err = parse("<% macro card title %><% end macro %>").expect_err("missing parens");
    assert!(err.message.contains("parentheses"), "message: {}", err.message);
}

#[test]
fn macro_requires_end_macro() {
    let err = parse("<% macro card() %>x").expect_err("unclosed macro");
    assert!(
        err.message.contains("expected `end macro`") || err.message.contains("code open"),
        "message: {}",
        err.message
    );
}

#[test]
fn macro_cannot_be_nested() {
    let err = parse("<% if ok %><% macro card() %>x<% end macro %><% end if %>").expect_err("nested macro");
    assert!(err.message.contains("top-level declaration"), "message: {}", err.message);
}

#[test]
fn macro_names_must_be_unique() {
    let err = parse("<% macro card() %>a<% end macro %><% macro card() %>b<% end macro %>")
        .expect_err("duplicate macro");
    assert!(err.message.contains("duplicate `macro` name"), "message: {}", err.message);
}

#[test]
fn macro_name_cannot_be_a_control_keyword() {
    let err = parse("<% macro if() %>a<% end macro %>").expect_err("keyword name");
    assert!(err.message.contains("control keyword"), "message: {}", err.message);
}

#[test]
fn macro_ir_round_trips_through_serde() {
    let doc = parse(r#"<% macro card(title, suffix = "!") %><% title %><% end macro %>"#).expect("parse");
    let json = serde_json::to_string(&doc).expect("serialize");
    let back: IrDocument = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(doc, back);
    assert!(json.contains("Stmt.Macro"), "json: {json}");
}
