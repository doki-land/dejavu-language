//! `break` / `continue` lexical binding contract at the parser boundary.

use dejavu_ir::types::{IrDocument, IrNode};
use dejavu_language::{ParseError, parse};
use dejavu_diagnostic::codes;

fn children(doc: &IrDocument) -> &[IrNode] {
    match &doc.body {
        IrNode::Template { children, .. } => children,
        other => panic!("expected Template body, got {other:?}"),
    }
}

fn control_code(err: &ParseError) -> &'static str {
    err.code.unwrap_or("none")
}

fn control_kind(err: &ParseError) -> &str {
    err.args
        .iter()
        .find(|(k, _)| k == "kind")
        .map(|(_, v)| v.as_str())
        .expect("control-outside-loop carries a `kind` arg")
}

fn assert_parse_fails(source: &str, kind: &str) {
    let err = parse(source).expect_err("control statement outside a loop must fail");
    assert_eq!(control_code(&err), codes::PARSE_CONTROL_OUTSIDE_LOOP);
    assert_eq!(control_kind(&err), kind);
}

#[test]
fn break_inside_loop_lowers_to_stmt_break() {
    let doc = parse("<% loop i in xs %><% break %><% end loop %>").expect("parse");
    let IrNode::StmtFor { body, .. } = &children(&doc)[0] else {
        panic!("expected Stmt.For");
    };
    assert!(matches!(body[0], IrNode::StmtBreak { .. }), "body: {body:?}");
}

#[test]
fn continue_inside_loop_lowers_to_stmt_continue() {
    let doc = parse("<% loop i in xs %><% continue %><% end loop %>").expect("parse");
    let IrNode::StmtFor { body, .. } = &children(&doc)[0] else {
        panic!("expected Stmt.For");
    };
    assert!(matches!(body[0], IrNode::StmtContinue { .. }), "body: {body:?}");
}

#[test]
fn break_inside_if_inside_loop_is_valid() {
    parse("<% loop i in xs %><% if i %><% break %><% end if %><% end loop %>")
        .expect("break binds through `if`");
}

#[test]
fn break_at_template_top_level_fails() {
    assert_parse_fails("<% break %>", "break");
}

#[test]
fn continue_at_template_top_level_fails() {
    assert_parse_fails("a<% continue %>b", "continue");
}

#[test]
fn break_in_loop_after_top_level_break_reports_first_escape() {
    assert_parse_fails("<% break %><% loop i in xs %><% break %><% end loop %>", "break");
}

#[test]
fn break_in_macro_body_without_loop_fails() {
    assert_parse_fails("<% macro m() %><% break %><% end macro %>", "break");
}

#[test]
fn break_in_loop_inside_macro_body_is_valid() {
    parse("<% macro m() %><% loop i in xs %><% break %><% end loop %><% end macro %>")
        .expect("loop inside the macro body binds the break");
}

#[test]
fn break_in_caller_block_without_own_loop_fails_across_call_site_loop() {
    assert_parse_fails(
        "<% loop i in xs %><% call m() %><% break %><% end call %><% end loop %>",
        "break",
    );
}

#[test]
fn break_in_loop_inside_caller_block_is_valid() {
    parse("<% loop i in xs %><% call m() %><% loop j in ys %><% break %><% end loop %><% end call %><% end loop %>")
        .expect("the caller block's own loop binds the break");
}

#[test]
fn break_in_block_body_without_loop_fails_across_enclosing_loop() {
    assert_parse_fails(
        "<% loop i in xs %><% block b %><% break %><% end block %><% end loop %>",
        "break",
    );
}

#[test]
fn break_in_partial_body_without_loop_fails() {
    assert_parse_fails("<% partial p %><% break %><% end partial %>", "break");
}

#[test]
fn break_as_object_key_stays_an_expression() {
    // Only a bare `break` code tag is the control statement; the word remains
    // usable as an object key inside expressions.
    parse("<% { break: 1 } %>").expect("`break` is only reserved as a statement head");
}

#[test]
fn tokens_after_bare_break_are_rejected() {
    let err = parse("<% loop i in xs %><% break.member %><% end loop %>")
        .expect_err("`break` must be the whole code tag");
    assert_eq!(control_code(&err), codes::PARSE_UNEXPECTED_TOKEN);
}
