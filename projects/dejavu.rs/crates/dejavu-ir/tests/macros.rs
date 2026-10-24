//! Macro declaration and invocation semantics in the shared renderer.

use dejavu_ir::{IrDocument, IrError, default_language, render_ir};
use serde_json::Value;

fn error_code(err: IrError) -> String {
    match err {
        IrError::Diagnostic(d) => d.code,
        other => panic!("expected a diagnostic, got {other}"),
    }
}

fn tpl(children: Vec<Value>) -> IrDocument {
    let doc_json = serde_json::json!({
        "irVersion": "1.0",
        "language": default_language(),
        "body": { "type": "Template", "children": children }
    });
    serde_json::from_value(doc_json).expect("parse ir json")
}

fn text(value: &str) -> Value {
    serde_json::json!({ "type": "Text", "value": value })
}

fn interp(expr: Value) -> Value {
    serde_json::json!({ "type": "Interpolation", "expression": expr, "trim": "none", "raw": false })
}

fn ident(name: &str) -> Value {
    serde_json::json!({ "type": "Expr.Identifier", "name": name })
}

fn literal(value: Value) -> Value {
    serde_json::json!({ "type": "Expr.Literal", "value": value })
}

fn param(name: &str, default: Option<Value>) -> Value {
    let mut node = serde_json::json!({ "name": name });
    if let Some(default) = default {
        node["default"] = default;
    }
    node
}

fn macro_node(name: &str, params: Vec<Value>, body: Vec<Value>) -> Value {
    serde_json::json!({
        "type": "Stmt.Macro",
        "name": name,
        "params": params,
        "body": body,
        "trim": "none",
    })
}

fn call(name: &str, args: Vec<Value>) -> Value {
    serde_json::json!({ "type": "Expr.Call", "callee": ident(name), "arguments": args })
}

#[test]
fn macro_declaration_emits_nothing_and_call_binds_parameters() {
    let doc = tpl(vec![
        macro_node(
            "card",
            vec![param("title", None), param("suffix", Some(literal(serde_json::json!("!"))))],
            vec![text("<b>"), interp(ident("title")), interp(ident("suffix")), text("</b>")],
        ),
        text("|"),
        interp(call("card", vec![literal(serde_json::json!("Hi"))])),
        interp(call("card", vec![literal(serde_json::json!("Yo")), literal(serde_json::json!("?"))])),
    ]);
    let out = render_ir(&doc, &serde_json::json!({})).expect("render");
    assert_eq!(out, "|<b>Hi!</b><b>Yo?</b>");
}

#[test]
fn macro_output_is_injected_without_double_escaping() {
    let doc = tpl(vec![
        macro_node("item", vec![param("title", None)], vec![text("<b>"), interp(ident("title")), text("</b>")]),
        interp(call("item", vec![literal(serde_json::json!("<i>"))])),
    ]);
    let out = render_ir(&doc, &serde_json::json!({})).expect("render");
    assert_eq!(out, "<b>&lt;i&gt;</b>");
}

#[test]
fn macro_body_does_not_inherit_the_caller_context() {
    let doc = tpl(vec![
        macro_node("show", vec![], vec![interp(ident("outer"))]),
        interp(call("show", vec![])),
    ]);
    let out = render_ir(&doc, &serde_json::json!({ "outer": "leak" })).expect("render");
    assert_eq!(out, "");
}

#[test]
fn macro_can_call_a_sibling_macro() {
    let doc = tpl(vec![
        macro_node("inner", vec![param("v", None)], vec![text("["), interp(ident("v")), text("]")]),
        macro_node("outer", vec![param("v", None)], vec![interp(call("inner", vec![ident("v")]))]),
        interp(call("outer", vec![literal(serde_json::json!("x"))])),
    ]);
    let out = render_ir(&doc, &serde_json::json!({})).expect("render");
    assert_eq!(out, "[x]");
}

#[test]
fn missing_macro_argument_is_a_render_error() {
    let doc = tpl(vec![
        macro_node("card", vec![param("title", None)], vec![interp(ident("title"))]),
        interp(call("card", vec![])),
    ]);
    let err = render_ir(&doc, &serde_json::json!({})).expect_err("missing argument");
    assert_eq!(error_code(err), "dejavu.template.missing-macro-argument");
}

#[test]
fn extra_macro_arguments_are_a_render_error() {
    let doc = tpl(vec![
        macro_node("card", vec![param("title", None)], vec![interp(ident("title"))]),
        interp(call("card", vec![literal(serde_json::json!("a")), literal(serde_json::json!("b"))])),
    ]);
    let err = render_ir(&doc, &serde_json::json!({})).expect_err("extra argument");
    assert_eq!(error_code(err), "dejavu.template.macro-arity");
}

#[test]
fn recursive_macro_hits_the_depth_limit() {
    let doc = tpl(vec![
        macro_node("again", vec![], vec![interp(call("again", vec![]))]),
        interp(call("again", vec![])),
    ]);
    let err = render_ir(&doc, &serde_json::json!({})).expect_err("recursion");
    assert_eq!(error_code(err), "dejavu.template.macro-recursion");
}

#[test]
fn macro_name_colliding_with_a_registered_function_is_rejected() {
    let doc = tpl(vec![macro_node("range", vec![], vec![text("x")])]);
    let err = render_ir(&doc, &serde_json::json!({})).expect_err("conflict");
    assert_eq!(error_code(err), "dejavu.template.macro-conflict");
}
