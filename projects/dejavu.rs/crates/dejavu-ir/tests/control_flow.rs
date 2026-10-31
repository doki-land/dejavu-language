//! `break` / `continue` runtime semantics of the JIT renderer.

use dejavu_ir::{IrDocument, codes, default_language, render_ir};
use serde_json::{Value, json};

fn doc(children: Vec<Value>) -> IrDocument {
    let doc_json = json!({
        "irVersion": "v0",
        "language": default_language(),
        "body": { "type": "Template", "children": children }
    });
    serde_json::from_value(doc_json).expect("ir json")
}

fn text(value: &str) -> Value {
    json!({ "type": "Text", "value": value })
}

fn ident(name: &str) -> Value {
    json!({ "type": "Expr.Identifier", "name": name })
}

fn member(object: Value, property: &str) -> Value {
    json!({ "type": "Expr.Member", "object": object, "property": property })
}

fn interp(expression: Value) -> Value {
    json!({ "type": "Interpolation", "expression": expression, "trim": "none", "raw": false })
}

fn stmt_for(item: &str, iterable: Value, body: Vec<Value>) -> Value {
    json!({ "type": "Stmt.For", "item": item, "iterable": iterable, "body": body, "trim": "none" })
}

fn literal(value: Value) -> Value {
    json!({ "type": "Expr.Literal", "value": value })
}

fn break_node() -> Value {
    json!({ "type": "Stmt.Break", "trim": "none" })
}

fn continue_node() -> Value {
    json!({ "type": "Stmt.Continue", "trim": "none" })
}

fn render(children: Vec<Value>, ctx: Value) -> String {
    render_ir(&doc(children), &ctx).expect("render")
}

#[test]
fn break_terminates_the_loop_and_keeps_partial_iteration_output() {
    let out = render(
        vec![stmt_for(
            "i",
            literal(json!([1, 2, 3])),
            vec![text("A"), break_node(), text("B")],
        )],
        json!({}),
    );
    assert_eq!(out, "A");
}

#[test]
fn continue_skips_the_rest_of_the_iteration() {
    let out = render(
        vec![stmt_for(
            "i",
            literal(json!([1, 2, 3])),
            vec![interp(ident("i")), continue_node(), text("X")],
        )],
        json!({}),
    );
    assert_eq!(out, "123");
}

#[test]
fn content_after_the_loop_still_renders() {
    let out = render(
        vec![
            stmt_for("i", literal(json!([1, 2])), vec![break_node()]),
            text("after"),
        ],
        json!({}),
    );
    assert_eq!(out, "after");
}

#[test]
fn nested_break_only_consumes_the_inner_loop() {
    let out = render(
        vec![stmt_for(
            "outer",
            literal(json!([1, 2])),
            vec![
                interp(ident("outer")),
                stmt_for(
                    "inner",
                    literal(json!([1, 2, 3])),
                    vec![text("-"), break_node(), text("never")],
                ),
            ],
        )],
        json!({}),
    );
    assert_eq!(out, "1-2-");
}

#[test]
fn forloop_metadata_is_restored_after_break() {
    // Inside the loop the metadata is visible; after the loop the `forloop`
    // slot must be gone again (non-strict: renders as empty string).
    let out = render(
        vec![
            stmt_for(
                "i",
                literal(json!([1, 2])),
                vec![interp(member(ident("forloop"), "index")), break_node()],
            ),
            interp(ident("forloop")),
        ],
        json!({}),
    );
    assert_eq!(out, "1");
}

#[test]
fn break_does_not_rewrite_last_flag() {
    // Breaking on the first iteration of a 3-element loop: `last` stays false
    // because metadata is computed from the original length.
    let out = render(
        vec![stmt_for(
            "i",
            literal(json!([1, 2, 3])),
            vec![interp(member(ident("forloop"), "last")), break_node()],
        )],
        json!({}),
    );
    assert_eq!(out, "false");
}

#[test]
fn continue_on_the_last_iteration_ends_the_loop_normally() {
    let out = render(
        vec![stmt_for(
            "i",
            literal(json!([7])),
            vec![interp(ident("i")), continue_node(), text("X")],
        )],
        json!({}),
    );
    assert_eq!(out, "7");
}

#[test]
fn stray_break_fails_with_control_outside_loop() {
    let err = render_ir(
        &doc(vec![break_node(), text("tail")]),
        &json!({}),
    )
    .expect_err("stray break must fail");
    let diag = match err {
        dejavu_ir::IrError::Diagnostic(d) => d,
        other => panic!("expected diagnostic, got {other:?}"),
    };
    assert_eq!(diag.code, codes::CONTROL_OUTSIDE_LOOP);
    assert_eq!(
        diag.args.get("kind").map(|v| format!("{v:?}")),
        Some("String(\"break\")".to_string())
    );
}

#[test]
fn stray_continue_fails_with_control_outside_loop() {
    let err = render_ir(&doc(vec![continue_node()]), &json!({})).expect_err("stray continue must fail");
    let diag = match err {
        dejavu_ir::IrError::Diagnostic(d) => d,
        other => panic!("expected diagnostic, got {other:?}"),
    };
    assert_eq!(diag.code, codes::CONTROL_OUTSIDE_LOOP);
    assert_eq!(
        diag.args.get("kind").map(|v| format!("{v:?}")),
        Some("String(\"continue\")".to_string())
    );
}

#[test]
fn break_inside_block_body_cannot_escape_to_enclosing_loop() {
    // Hand-authored IR: the renderer treats a block body as its own unit.
    let block = json!({
        "type": "Stmt.Block", "name": "b", "body": [break_node()], "trim": "none"
    });
    let err = render_ir(
        &doc(vec![stmt_for("i", literal(json!([1, 2])), vec![block])]),
        &json!({}),
    )
    .expect_err("block body is a separate lexical unit");
    let diag = match err {
        dejavu_ir::IrError::Diagnostic(d) => d,
        other => panic!("expected diagnostic, got {other:?}"),
    };
    assert_eq!(diag.code, codes::CONTROL_OUTSIDE_LOOP);
}
