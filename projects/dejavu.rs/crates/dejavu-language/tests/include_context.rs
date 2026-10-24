//! `include` context modifiers at the parser/IR boundary:
//! `with <expr>`, `only`, and `without context`.

use dejavu_ir::types::{IrDocument, IrNode};
use dejavu_language::parse;

fn include_node(source: &str) -> IrNode {
    let doc: IrDocument = parse(source).expect("parse");
    let IrNode::Template { children, .. } = doc.body else {
        panic!("expected Template body");
    };
    children.into_iter().find(|c| matches!(c, IrNode::StmtInclude { .. })).expect("Stmt.Include")
}

#[test]
fn plain_include_inherits_context() {
    let IrNode::StmtInclude { with, only, .. } = include_node(r#"<% include "partial.dejavu" %>"#) else {
        unreachable!()
    };
    assert!(with.is_none());
    assert!(!only);
}

#[test]
fn include_with_expression_merges_context() {
    let IrNode::StmtInclude { path, with, only, .. } =
        include_node(r#"<% include "partial.dejavu" with card %>"#)
    else {
        unreachable!()
    };
    assert!(matches!(path.as_ref(), IrNode::ExprLiteral { .. }));
    assert!(matches!(with.as_deref(), Some(IrNode::ExprIdentifier { name, .. }) if name == "card"));
    assert!(!only);
}

#[test]
fn include_with_path_expression_is_not_consumed_by_modifier() {
    let IrNode::StmtInclude { path, with, only, .. } =
        include_node("<% include page.partial with page.slot %>")
    else {
        unreachable!()
    };
    assert!(matches!(path.as_ref(), IrNode::ExprMember { property, .. } if property == "partial"));
    assert!(matches!(with.as_deref(), Some(IrNode::ExprMember { property, .. }) if property == "slot"));
    assert!(!only);
}

#[test]
fn include_only_isolates_context() {
    let IrNode::StmtInclude { with, only, .. } = include_node(r#"<% include "p" only %>"#) else {
        unreachable!()
    };
    assert!(with.is_none());
    assert!(only);
}

#[test]
fn include_without_context_isolates_context() {
    let IrNode::StmtInclude { with, only, .. } = include_node(r#"<% include "p" without context %>"#)
    else {
        unreachable!()
    };
    assert!(with.is_none());
    assert!(only);
}

#[test]
fn include_with_expression_only() {
    let IrNode::StmtInclude { with, only, .. } = include_node(r#"<% include "p" with card only %>"#)
    else {
        unreachable!()
    };
    assert!(matches!(with.as_deref(), Some(IrNode::ExprIdentifier { name, .. }) if name == "card"));
    assert!(only);
}

#[test]
fn malformed_include_modifiers_are_rejected() {
    for source in [
        r#"<% include "p" only trailing %>"#,
        r#"<% include "p" without %>"#,
        r#"<% include "p" without context trailing %>"#,
        r#"<% include "p" with %>"#,
        r#"<% include %>"#,
    ] {
        let err = parse(source).expect_err("malformed include must be rejected");
        assert!(
            err.message.contains("include")
                || err.message.contains("with")
                || err.message.contains("without")
                || err.message.contains("unexpected tokens after"),
            "unexpected message for `{source}`: {}",
            err.message
        );
    }
}

#[test]
fn include_ir_round_trips_through_serde() {
    let doc = parse(r#"<% include "p" with card only %>"#).expect("parse");
    let json = serde_json::to_string(&doc).expect("serialize");
    let back: IrDocument = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(doc, back);
}
