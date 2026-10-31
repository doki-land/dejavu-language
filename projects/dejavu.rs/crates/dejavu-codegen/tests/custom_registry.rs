//! Compile-time behaviour of host-supplied `filter` / `function` / `test`
//! registry entries.
//!
//! Built-in names always win; only a name the built-in tables reject as
//! `Unknown` falls through to `options.customRegistry`. The emitted call uses
//! the canonical name and forwards the render-time `__custom` object, so the
//! generated module never depends on module-level state.

use dejavu_codegen::{PrecompileResult, precompile_single};
use dejavu_diagnostic::{ArgValue, Stage};
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn compile(ir: &Value, options: &Value) -> PrecompileResult {
    precompile_single(&ir.to_string(), Some(&options.to_string())).expect("compile should succeed")
}

fn options(custom_registry: Value) -> Value {
    json!({ "customRegistry": custom_registry })
}

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

fn interp(expression: Value) -> Value {
    json!({ "type": "Interpolation", "expression": expression, "trim": "none", "raw": false })
}

fn ident(name: &str) -> Value {
    json!({ "type": "Expr.Identifier", "name": name })
}

fn pipe(filter: &str, expression: Value, args: Vec<Value>) -> Value {
    json!({ "type": "Expr.Pipe", "filter": filter, "expression": expression, "arguments": args })
}

fn call(name: &str, args: Vec<Value>) -> Value {
    json!({ "type": "Expr.Call", "callee": ident(name), "arguments": args })
}

fn test_node(expression: Value, test: &str) -> Value {
    json!({
        "type": "Expr.Test", "expression": expression, "test": test,
        "arguments": [], "negated": false
    })
}

fn entry(namespace: &str, name: &str, aliases: &[&str], min: u32, max: u32, capability: &str) -> Value {
    json!({
        "namespace": namespace,
        "name": name,
        "aliases": aliases,
        "minArgs": min,
        "maxArgs": max,
        "capability": capability
    })
}

fn round_body(s: &str) -> String {
    let body = s
        .lines()
        .skip_while(|l| l.trim_start().starts_with("import ") || l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    body.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ---------------------------------------------------------------------------
// Filter
// ---------------------------------------------------------------------------

#[test]
fn custom_filter_compiles_and_renders() {
    let ir = doc(vec![text("filter="), interp(pipe("shout", ident("name"), vec![]))]);
    let res = compile(
        &ir,
        &options(json!([entry("filter", "shout", &[], 0, 0, "pure")])),
    );
    assert!(res.diagnostics.is_empty(), "{:?}", res.diagnostics);
    let body = round_body(&res.code);
    assert!(
        body.contains("applyFilter(\"shout\", ctx.name, [], __custom)"),
        "{body}"
    );

    let node_stub = r#"
const __rt = {
  htmlEscape: (s) => String(s).replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c])),
  valueToString: (v) => (v == null) ? '' : String(v),
  applyFilter: (name, v, args, custom) => {
    const impl = custom && custom.filter && custom.filter[name];
    if (impl) return impl(v, args);
    return v;
  },
  runtimeError: (code, detail) => new Error(detail ? `${code}: ${detail}` : code),
};
const { htmlEscape, valueToString, applyFilter, runtimeError } = __rt;
"#;
    let mut lines: Vec<&str> = res.code.lines().collect();
    lines.retain(|l| !l.trim().starts_with("import ") && !l.trim().starts_with("export const "));
    let body = lines.join("\n").replace("export function render", "function render");
    let render_call = "console.log(render({ name: 'dejavu' }, { custom: { filter: { shout: (v) => String(v) + '!' } } }));";
    let script = format!("{node_stub}\n{body}\n{render_call}");
    match std::process::Command::new("node").args(["-e", &script]).output() {
        Ok(out) if out.status.success() => {
            assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "filter=dejavu!");
        }
        Ok(out) => panic!("node failed: {}", String::from_utf8_lossy(&out.stderr)),
        Err(_) => eprintln!("SKIP: node not available"),
    }
}

#[test]
fn custom_filter_alias_emits_canonical_name() {
    let ir = doc(vec![interp(pipe("loud", ident("name"), vec![]))]);
    let res = compile(
        &ir,
        &options(json!([entry("filter", "shout", &["loud"], 0, 0, "pure")])),
    );
    assert!(res.diagnostics.is_empty(), "{:?}", res.diagnostics);
    let body = round_body(&res.code);
    assert!(body.contains("applyFilter(\"shout\", ctx.name, [], __custom)"), "{body}");
    assert!(!body.contains("applyFilter(\"loud\""), "{body}");
}

#[test]
fn custom_filter_arity_is_a_compile_diagnostic() {
    let ir = doc(vec![interp(pipe("shout", ident("name"), vec![]))]);
    let res = compile(
        &ir,
        &options(json!([entry("filter", "shout", &[], 1, 2, "pure")])),
    );
    let diagnostic = res.diagnostics.first().expect("expected a diagnostic");
    assert_eq!(diagnostic.code, "dejavu.template.filter-arity");
    assert_eq!(diagnostic.stage, Stage::Compile);
    assert_eq!(diagnostic.args["filter"], ArgValue::String("shout".into()));
    assert_eq!(diagnostic.args["minArgs"], ArgValue::Number(1.0));
    assert_eq!(diagnostic.args["maxArgs"], ArgValue::Number(2.0));
    assert_eq!(diagnostic.args["got"], ArgValue::Number(0.0));
}

#[test]
fn custom_safe_output_filter_skips_auto_escape() {
    let ir = doc(vec![interp(pipe("bolder", ident("markup"), vec![]))]);
    let res = compile(
        &ir,
        &options(json!([entry("filter", "bolder", &[], 0, 0, "safe-output")])),
    );
    assert!(res.diagnostics.is_empty(), "{:?}", res.diagnostics);
    let body = round_body(&res.code);
    assert!(body.contains("__out.push(valueToString(__v));"), "{body}");
    assert!(!body.contains("htmlEscape(valueToString(__v))"), "{body}");
}

#[test]
fn custom_capability_denial_is_a_compile_diagnostic() {
    let ir = doc(vec![interp(pipe("bolder", ident("markup"), vec![]))]);
    let res = compile(
        &ir,
        &json!({
            "capabilities": ["pure"],
            "customRegistry": [entry("filter", "bolder", &[], 0, 0, "safe-output")]
        }),
    );
    let diagnostic = res.diagnostics.first().expect("expected a diagnostic");
    assert_eq!(diagnostic.code, "dejavu.template.capability-not-allowed");
    assert_eq!(diagnostic.stage, Stage::Compile);
    assert_eq!(diagnostic.args["name"], ArgValue::String("bolder".into()));
    assert_eq!(diagnostic.args["capability"], ArgValue::String("safe-output".into()));
}

#[test]
fn custom_unknown_name_still_reports_unknown_filter() {
    let ir = doc(vec![interp(pipe("absent", ident("name"), vec![]))]);
    let res = compile(&ir, &options(json!([])));
    assert_eq!(res.diagnostics.first().unwrap().code, "dejavu.template.unknown-filter");
}

// ---------------------------------------------------------------------------
// Function / test
// ---------------------------------------------------------------------------

#[test]
fn custom_function_and_test_use_canonical_names() {
    let func_alias = doc(vec![interp(call("dbl", vec![json!({ "type": "Expr.Literal", "value": 21 })]))]);
    let res = compile(
        &func_alias,
        &options(json!([
            entry("function", "twice", &["dbl"], 1, 1, "pure"),
            entry("test", "evenish", &[], 0, 0, "pure")
        ])),
    );
    assert!(res.diagnostics.is_empty(), "{:?}", res.diagnostics);
    let body = round_body(&res.code);
    assert!(body.contains("callFunction(\"twice\", [21], __custom)"), "{body}");

    let ir = doc(vec![interp(test_node(ident("n"), "evenish"))]);
    let res = compile(
        &ir,
        &options(json!([entry("test", "evenish", &["fair"], 0, 0, "pure")])),
    );
    assert!(res.diagnostics.is_empty(), "{:?}", res.diagnostics);
    let body = round_body(&res.code);
    assert!(body.contains("applyTest(\"evenish\", ctx.n, [], false, __custom)"), "{body}");
}

#[test]
fn custom_function_arity_is_a_compile_diagnostic() {
    let ir = doc(vec![interp(call("twice", vec![]))]);
    let res = compile(
        &ir,
        &options(json!([entry("function", "twice", &[], 1, 1, "pure")])),
    );
    let diagnostic = res.diagnostics.first().expect("expected a diagnostic");
    assert_eq!(diagnostic.code, "dejavu.template.function-arity");
    assert_eq!(diagnostic.stage, Stage::Compile);
    assert_eq!(diagnostic.args["function"], ArgValue::String("twice".into()));
}

#[test]
fn custom_test_arity_is_a_compile_diagnostic() {
    let ir = doc(vec![json!({
        "type": "Interpolation",
        "expression": {
            "type": "Expr.Test",
            "expression": ident("n"),
            "test": "evenish",
            "arguments": [{ "type": "Expr.Literal", "value": 1 }],
            "negated": false
        },
        "trim": "none",
        "raw": false
    })]);
    let res = compile(
        &ir,
        &options(json!([entry("test", "evenish", &[], 0, 0, "pure")])),
    );
    let diagnostic = res.diagnostics.first().expect("expected a diagnostic");
    assert_eq!(diagnostic.code, "dejavu.template.test-arity");
    assert_eq!(diagnostic.stage, Stage::Compile);
    assert_eq!(diagnostic.args["test"], ArgValue::String("evenish".into()));
}

// ---------------------------------------------------------------------------
// Registry validation
// ---------------------------------------------------------------------------

#[test]
fn conflicting_entry_is_a_compile_diagnostic() {
    // `upper` is a built-in filter alias, so registering it is a conflict.
    let ir = doc(vec![text("x")]);
    let res = compile(
        &ir,
        &options(json!([entry("filter", "upper", &[], 0, 0, "pure")])),
    );
    let diagnostic = res.diagnostics.first().expect("expected a diagnostic");
    assert_eq!(diagnostic.code, "dejavu.template.registry-conflict");
    assert_eq!(diagnostic.stage, Stage::Compile);
    assert_eq!(diagnostic.args["namespace"], ArgValue::String("filter".into()));
    assert_eq!(diagnostic.args["name"], ArgValue::String("upper".into()));
}

#[test]
fn duplicate_custom_spellings_are_a_compile_diagnostic() {
    let ir = doc(vec![text("x")]);
    let res = compile(
        &ir,
        &options(json!([
            entry("filter", "shout", &["loud"], 0, 0, "pure"),
            entry("filter", "holler", &["loud"], 0, 0, "pure")
        ])),
    );
    let diagnostic = res.diagnostics.first().expect("expected a diagnostic");
    assert_eq!(diagnostic.code, "dejavu.template.registry-conflict");
    assert_eq!(diagnostic.stage, Stage::Compile);
}
