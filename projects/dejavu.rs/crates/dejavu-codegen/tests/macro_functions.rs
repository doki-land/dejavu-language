//! AOT emission for template macros: hoisted functions and safe output.

use dejavu_codegen::precompile_single;
use serde_json::{Value, json};

fn doc(children: Vec<Value>) -> Value {
    json!({
        "irVersion": "1.0",
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

fn macro_card() -> Value {
    json!({
        "type": "Stmt.Macro",
        "name": "card",
        "params": [
            { "name": "title" },
            { "name": "suffix", "default": { "type": "Expr.Literal", "value": "!" } }
        ],
        "body": [
            { "type": "Text", "value": "<b>" },
            {
                "type": "Interpolation",
                "expression": { "type": "Expr.Identifier", "name": "title" },
                "trim": "none",
                "raw": false
            },
            { "type": "Text", "value": "</b>" }
        ],
        "trim": "none"
    })
}

fn call_card() -> Value {
    json!({
        "type": "Interpolation",
        "expression": {
            "type": "Expr.Call",
            "callee": { "type": "Expr.Identifier", "name": "card" },
            "arguments": [{ "type": "Expr.Literal", "value": "Hi" }]
        },
        "trim": "none",
        "raw": false
    })
}

#[test]
fn macro_declaration_emits_a_hoisted_module_function() {
    let result = precompile_single(&doc(vec![macro_card(), call_card()]).to_string(), None).expect("compile");
    assert!(result.diagnostics.is_empty(), "diagnostics: {:?}", result.diagnostics);
    let code = &result.code;
    assert!(
        code.contains("function __macro_card(__invokeCtx, args, __depth, __strict, __caller)"),
        "code:\n{code}"
    );
    assert!(code.contains("dejavu.template.missing-macro-argument"), "code:\n{code}");
    assert!(code.contains("dejavu.template.macro-arity"), "code:\n{code}");
    assert!(code.contains("__macro_card(ctx, [\"Hi\"], 1, __strict, null)"), "code:\n{code}");
}

#[test]
fn macro_call_output_is_not_auto_escaped() {
    let result = precompile_single(&doc(vec![macro_card(), call_card()]).to_string(), None).expect("compile");
    let code = &result.code;
    let call_line = code
        .lines()
        .find(|l| l.contains("__macro_card(ctx, [\"Hi\"]"))
        .expect("macro call line");
    assert!(!call_line.contains("htmlEscape"), "line: {call_line}");
}

fn macro_wrap() -> Value {
    json!({
        "type": "Stmt.Macro",
        "name": "wrap",
        "params": [],
        "body": [
            { "type": "Text", "value": "<div>" },
            {
                "type": "Interpolation",
                "expression": {
                    "type": "Expr.Call",
                    "callee": { "type": "Expr.Identifier", "name": "caller" },
                    "arguments": []
                },
                "trim": "none",
                "raw": false
            },
            { "type": "Text", "value": "</div>" }
        ],
        "trim": "none"
    })
}

fn call_wrap() -> Value {
    json!({
        "type": "Stmt.Call",
        "name": "wrap",
        "arguments": [],
        "body": [
            { "type": "Text", "value": "inner" }
        ],
        "trim": "none"
    })
}

#[test]
fn call_block_emits_a_caller_closure() {
    let result = precompile_single(&doc(vec![macro_wrap(), call_wrap()]).to_string(), None)
        .expect("compile");
    assert!(result.diagnostics.is_empty(), "diagnostics: {:?}", result.diagnostics);
    let code = &result.code;
    assert!(code.contains("__macro_wrap(ctx, [], 1, __strict, function () {"), "code:\n{code}");
    assert!(code.contains("const __out = [];"), "code:\n{code}");
    assert!(code.contains("__caller()"), "code:\n{code}");
}

#[test]
fn caller_outside_a_macro_is_a_compile_error() {
    let call = json!({
        "type": "Interpolation",
        "expression": {
            "type": "Expr.Call",
            "callee": { "type": "Expr.Identifier", "name": "caller" },
            "arguments": []
        },
        "trim": "none",
        "raw": false
    });
    let result = precompile_single(&doc(vec![call]).to_string(), None).expect("compile");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == "dejavu.template.caller-unavailable"),
        "diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn call_to_an_unknown_macro_is_a_compile_error() {
    let call = json!({
        "type": "Stmt.Call",
        "name": "missing",
        "arguments": [],
        "body": [],
        "trim": "none"
    });
    let result = precompile_single(&doc(vec![call]).to_string(), None).expect("compile");
    assert!(
        result.diagnostics.iter().any(|d| d.code == "dejavu.template.unknown-function"),
        "diagnostics: {:?}",
        result.diagnostics
    );
}
