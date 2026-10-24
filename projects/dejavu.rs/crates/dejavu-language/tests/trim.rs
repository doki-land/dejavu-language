//! Whitespace-control contract at the parser/IR boundary.

use dejavu_ir::types::{IrDocument, IrNode};
use dejavu_language::parse;

fn children(doc: &IrDocument) -> &[IrNode] {
    match &doc.body {
        IrNode::Template { children, .. } => children,
        other => panic!("expected Template body, got {other:?}"),
    }
}

fn trim_of(node: &IrNode) -> &str {
    match node {
        IrNode::Interpolation { trim, .. }
        | IrNode::StmtIf { trim, .. }
        | IrNode::StmtFor { trim, .. }
        | IrNode::StmtBlock { trim, .. }
        | IrNode::StmtInclude { trim, .. } => trim,
        other => panic!("node has no trim: {other:?}"),
    }
}

fn text_of(node: &IrNode) -> Option<&str> {
    match node {
        IrNode::Text { value, .. } => Some(value),
        _ => None,
    }
}

#[test]
fn left_marker_is_recorded_on_the_tag() {
    let doc = parse("a\n  <%_ x %>").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[0]), Some("a\n  "));
    assert_eq!(trim_of(&nodes[1]), "ws");
}

#[test]
fn left_levels_map_to_expected_trim_values() {
    for (marker, level) in [('.', "none"), ('_', "ws"), ('-', "nl"), ('~', "ws_nl"), ('=', "all")] {
        let source = format!("<%{marker} x %>");
        let doc = parse(&source).expect("parse");
        let nodes = children(&doc);
        assert_eq!(trim_of(&nodes[0]), level, "marker `{marker}`");
    }
}

#[test]
fn right_marker_trims_the_following_text() {
    let doc = parse("A\n<% x -%>\nB").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[0]), Some("A\n"));
    assert_eq!(trim_of(&nodes[1]), "none");
    // A single line break after the tag is removed, the rest is kept.
    assert_eq!(text_of(&nodes[2]), Some("B"));
}

#[test]
fn right_ws_nl_marker_removes_indent_and_one_break() {
    let doc = parse("<% x ~%>\n  B").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[1]), Some("B"));
}

#[test]
fn right_all_marker_removes_repeated_breaks() {
    let doc = parse("<% x =%>\n\n\nB").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[1]), Some("B"));
}

#[test]
fn right_marker_on_block_open_trims_body_prefix() {
    let doc = parse("<% if x -%>\n  yes\n<% end if %>").expect("parse");
    let nodes = children(&doc);
    let IrNode::StmtIf { consequent, .. } = &nodes[0] else {
        panic!("expected Stmt.If");
    };
    assert_eq!(text_of(&consequent[0]), Some("  yes\n"));
}

#[test]
fn markers_on_closers_are_rejected() {
    for source in [
        "<% if x %>a<% else -%>b<% end if %>",
        "<% if x %>a<% end if -%>",
        "<% loop i in xs %>a<% end loop_%>",
    ] {
        let err = parse(source).expect_err("closer markers must be rejected");
        assert!(
            err.message.contains("whitespace control markers"),
            "unexpected message for `{source}`: {}",
            err.message
        );
    }
}

#[test]
fn plain_tags_keep_the_escaped_literal_path() {
    let doc = parse("a<%!b").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[1]), Some("<%"));
}
