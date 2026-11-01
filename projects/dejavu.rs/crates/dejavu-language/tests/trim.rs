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

#[test]
fn standalone_directive_line_is_removed() {
    let doc = parse("A\n  <% include \"p\" %>\nB").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[0]), Some("A\n"));
    assert_eq!(text_of(&nodes[2]), Some("B"));
}

#[test]
fn standalone_if_else_chain_removes_every_control_line() {
    let doc = parse("A\n<% if x %>\nB\n<% else %>\nC\n<% end if %>\nD").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[0]), Some("A\n"));
    let IrNode::StmtIf { consequent, alternate, .. } = &nodes[1] else {
        panic!("expected StmtIf");
    };
    assert_eq!(text_of(&consequent[0]), Some("B\n"));
    let alternate = alternate.as_ref().expect("alternate branch");
    assert_eq!(text_of(&alternate[0]), Some("C\n"));
    assert_eq!(text_of(&nodes[2]), Some("D"));
}

#[test]
fn standalone_closer_keeps_next_line_indent() {
    let doc = parse("<% if x %>\nB\n  <% end if %>\n  C").expect("parse");
    let nodes = children(&doc);
    let IrNode::StmtIf { consequent, .. } = &nodes[0] else {
        panic!("expected StmtIf");
    };
    assert_eq!(text_of(&consequent[0]), Some("B\n"));
    assert_eq!(text_of(&nodes[1]), Some("  C"));
}

#[test]
fn expression_slot_is_never_standalone() {
    let doc = parse("A\n<% x %>\nB").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[0]), Some("A\n"));
    assert_eq!(text_of(&nodes[2]), Some("\nB"));
}

#[test]
fn explicit_markers_suppress_standalone_on_their_side() {
    // Left marker `<%_` owns the left side: the parse-time text keeps the
    // indentation (the marker strips it at render), standalone still removes
    // the line break after the tag.
    let doc = parse("A\n  <%_ include \"p\" %>\nB").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[0]), Some("A\n  "));
    assert_eq!(text_of(&nodes[2]), Some("B"));
    // Right marker `_%>` owns the right side: only horizontal whitespace is
    // stripped, the standalone line-break removal is suppressed.
    let doc = parse("A\n<% include \"p\" _%>\nB").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[2]), Some("\nB"));
}

#[test]
fn standalone_right_requires_the_rest_of_the_line_to_be_blank() {
    let doc = parse("A\n<% include \"p\" %>B").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[2]), Some("B"));
    let doc = parse("A\n<% include \"p\" %>   \nB").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[0]), Some("A\n"));
    assert_eq!(text_of(&nodes[2]), Some("B"));
}

#[test]
fn standalone_left_requires_only_indent_before_the_tag() {
    let doc = parse("A <% include \"p\" %>\nB").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[0]), Some("A "));
    // The right side still qualifies: the tag ends its own line.
    assert_eq!(text_of(&nodes[2]), Some("B"));
}

#[test]
fn standalone_handles_crlf_and_eof_edges() {
    let doc = parse("A\r\n<% include \"p\" %>\r\nB").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[0]), Some("A\r\n"));
    assert_eq!(text_of(&nodes[2]), Some("B"));
    let doc = parse("<% include \"p\" %>\n").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[1]), Some(""));
}

#[test]
fn standalone_comment_line_is_removed() {
    let doc = parse("A\n  <# note #>\nB").expect("parse");
    let nodes = children(&doc);
    assert_eq!(text_of(&nodes[0]), Some("A\n"));
    assert!(matches!(&nodes[1], IrNode::Comment { .. }));
    assert_eq!(text_of(&nodes[2]), Some("B"));
}

#[test]
fn standalone_loop_body_and_closers() {
    let doc = parse("<% loop i in xs %>\nA\n<% end loop %>\nB").expect("parse");
    let nodes = children(&doc);
    let IrNode::StmtFor { body, .. } = &nodes[0] else {
        panic!("expected StmtFor");
    };
    assert_eq!(text_of(&body[0]), Some("A\n"));
    assert_eq!(text_of(&nodes[1]), Some("B"));
}
