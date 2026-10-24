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
    assert!(code.contains("function __macro_card(__caller, args, __depth, __strict)"), "code:\n{code}");
    assert!(code.contains("dejavu.template.missing-macro-argument"), "code:\n{code}");
    assert!(code.contains("dejavu.template.macro-arity"), "code:\n{code}");
    assert!(code.contains("__macro_card(ctx, [\"Hi\"], 1, __strict)"), "code:\n{code}");
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
