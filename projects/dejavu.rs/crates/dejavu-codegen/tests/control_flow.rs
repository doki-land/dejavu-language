//! AOT emit of `break` / `continue`: raw JS control inside a bound loop,
//! defensive runtime rejection for hand-authored IR that escapes its unit.

use dejavu_codegen::precompile_single;
use serde_json::{Value, json};

fn doc(children: Vec<Value>) -> Value {
    json!({
        "irVersion": "v0",
        "language": {
            "syntaxMode": "template",
            "template": {
                "codeStart": "<%", "codeEnd": "%>",
                "commentStart": "<#", "commentEnd": "#>",
                "supportFilterPipe": true, "legacyFor": false
            }
        },
        "body": { "type": "Template", "children": children }
    })
}

fn text(value: &str) -> Value {
    json!({ "type": "Text", "value": value })
}

fn literal(value: Value) -> Value {
    json!({ "type": "Expr.Literal", "value": value })
}

fn ident(name: &str) -> Value {
    json!({ "type": "Expr.Identifier", "name": name })
}

fn stmt_for(item: &str, iterable: Value, body: Vec<Value>) -> Value {
    json!({ "type": "Stmt.For", "item": item, "iterable": iterable, "body": body, "trim": "none" })
}

fn break_node() -> Value {
    json!({ "type": "Stmt.Break", "trim": "none" })
}

fn continue_node() -> Value {
    json!({ "type": "Stmt.Continue", "trim": "none" })
}

fn compile(ir: &Value) -> String {
    precompile_single(&serde_json::to_string(ir).unwrap(), None)
        .expect("compile")
        .code
}

#[test]
fn break_inside_loop_emits_js_break() {
    let code = compile(&doc(vec![stmt_for("i", literal(json!([1, 2])), vec![break_node()])]));
    assert!(code.contains("break;\n"), "code: {code}");
    assert!(!code.contains("control-outside-loop"), "code: {code}");
}

#[test]
fn continue_inside_loop_emits_js_continue() {
    let code = compile(&doc(vec![stmt_for("i", literal(json!([1, 2])), vec![continue_node()])]));
    assert!(code.contains("continue;\n"), "code: {code}");
    assert!(!code.contains("control-outside-loop"), "code: {code}");
}

#[test]
fn nested_loop_break_binds_the_inner_js_loop() {
    let inner = stmt_for("j", literal(json!([1])), vec![break_node()]);
    let code = compile(&doc(vec![stmt_for("i", literal(json!([1, 2])), vec![inner])]));
    // One raw `break;` and no defensive throw: the inner loop binds it.
    assert_eq!(code.matches("break;").count(), 1, "code: {code}");
    assert!(!code.contains("control-outside-loop"), "code: {code}");
}

#[test]
fn stray_break_at_module_level_emits_defensive_throw() {
    let code = compile(&doc(vec![break_node(), text("tail")]));
    assert!(code.contains("dejavu.template.control-outside-loop"), "code: {code}");
    assert!(code.contains("kind: \"break\""), "code: {code}");
    assert!(!code.contains("break;\n"), "must not emit raw JS break: {code}");
}

#[test]
fn caller_block_body_is_a_fresh_control_scope() {
    // Hand-authored IR: a stray break inside the caller block must not bind
    // to the enclosing JS loop (the caller closure is a function boundary).
    let mac = json!({
        "type": "Stmt.Macro", "name": "m", "params": [], "body": [text("M")], "trim": "none"
    });
    let call = json!({
        "type": "Stmt.Call", "name": "m", "arguments": [], "params": [],
        "body": [break_node()], "trim": "none"
    });
    let code = compile(&doc(vec![mac, stmt_for("i", literal(json!([1, 2])), vec![call])]));
    assert!(code.contains("dejavu.template.control-outside-loop"), "code: {code}");
}

#[test]
fn break_inside_loop_inside_caller_block_emits_js_break() {
    let mac = json!({
        "type": "Stmt.Macro", "name": "m", "params": [], "body": [text("M")], "trim": "none"
    });
    let inner = stmt_for("j", literal(json!([1])), vec![break_node()]);
    let call = json!({
        "type": "Stmt.Call", "name": "m", "arguments": [], "params": [],
        "body": [inner], "trim": "none"
    });
    let code = compile(&doc(vec![mac, stmt_for("i", literal(json!([1, 2])), vec![call])]));
    assert!(code.contains("break;\n"), "code: {code}");
    assert!(!code.contains("control-outside-loop"), "code: {code}");
}

#[test]
fn macro_body_loop_emits_js_break_inside_hoisted_function() {
    let inner = stmt_for("j", literal(json!([1])), vec![break_node()]);
    let mac = json!({
        "type": "Stmt.Macro", "name": "m", "params": [], "body": [inner], "trim": "none"
    });
    let call = json!({
        "type": "Expr.Call", "callee": ident("m"), "arguments": []
    });
    let interp = json!({ "type": "Interpolation", "expression": call, "trim": "none", "raw": true });
    let code = compile(&doc(vec![mac, interp]));
    assert!(code.contains("break;\n"), "code: {code}");
    assert!(!code.contains("control-outside-loop"), "code: {code}");
}
