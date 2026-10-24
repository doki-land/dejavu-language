//! End-to-end single-template codegen tests.
//!
//! Strategy:
//!
//! 1. Hand-write minimal IR JSON fixtures (covers Text, Interpolation with
//!    Pipe, If/ElseIf, For with index, Expr.Member, Expr.Index, Expr.Binary,
//!    Expr.Call, strictUndefined, JS reserved-word identifier).
//! 2. Call [`precompile_single`].
//! 3. Assert structural facts using whitespace-normalised substrings so
//!    indentation / line-break choices in [`crate::codegen::build_module`]
//!    remain flexible.
//! 4. Run the generated JS under a tiny Node.js helper stub via
//!    `node -e` subprocess (skipped gracefully if node is absent).

use dejavu_codegen::{LineOrigin, PrecompileResult, precompile_single};
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Collapse all whitespace into single spaces.
fn round(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `round` of everything after the `import` lines — useful when we want to
/// inspect whether runtime helpers are *called*, not just *imported*.
fn round_body(s: &str) -> String {
    let body = s
        .lines()
        .skip_while(|l| l.trim_start().starts_with("import ") || l.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    round(&body)
}

fn compile(ir_json: &str) -> PrecompileResult {
    precompile_single(ir_json, None).expect("compile should succeed")
}

fn compile_opts(ir_json: &str, opts_json: &str) -> PrecompileResult {
    precompile_single(ir_json, Some(opts_json)).expect("compile should succeed")
}

// ---------------------------------------------------------------------------
// Fixture builders
// ---------------------------------------------------------------------------

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

fn text(value: &str) -> Value {
    json!({ "type": "Text", "value": value })
}

fn comment(value: &str) -> Value {
    json!({ "type": "Comment", "value": value })
}

fn interp(expression: Value, raw: bool) -> Value {
    json!({ "type": "Interpolation", "expression": expression, "trim": "none", "raw": raw })
}

fn ident(name: &str) -> Value { json!({ "type": "Expr.Identifier", "name": name }) }

fn lit(value: Value) -> Value { json!({ "type": "Expr.Literal", "value": value }) }

fn member(object: Value, property: &str) -> Value {
    json!({ "type": "Expr.Member", "object": object, "property": property })
}

fn index(object: Value, idx: Value) -> Value {
    json!({ "type": "Expr.Index", "object": object, "index": idx })
}

fn binary(op: &str, l: Value, r: Value) -> Value {
    json!({ "type": "Expr.Binary", "operator": op, "left": l, "right": r })
}

fn unary(op: &str, a: Value) -> Value {
    json!({ "type": "Expr.Unary", "operator": op, "argument": a })
}

fn pipe(filter: &str, expression: Value, args: Vec<Value>) -> Value {
    json!({ "type": "Expr.Pipe", "filter": filter, "expression": expression, "arguments": args })
}

fn call(callee: Value, args: Vec<Value>) -> Value {
    json!({ "type": "Expr.Call", "callee": callee, "arguments": args })
}

fn test_node(expression: Value, test: &str, args: Vec<Value>, negated: bool) -> Value {
    json!({
        "type": "Expr.Test", "expression": expression, "test": test,
        "arguments": args, "negated": negated
    })
}

fn stmt_if(
    test: Value,
    consequent: Vec<Value>,
    else_ifs: Vec<Value>,
    alternate: Option<Vec<Value>>,
) -> Value {
    let mut obj = json!({
        "type": "Stmt.If", "test": test,
        "consequent": consequent, "elseIfs": else_ifs, "trim": "none"
    });
    if let Some(alt) = alternate { obj["alternate"] = json!(alt); }
    obj
}

fn stmt_elseif(test: Value, body: Vec<Value>) -> Value {
    json!({ "type": "Stmt.ElseIf", "test": test, "consequent": body, "trim": "none" })
}

fn stmt_for(item: &str, index: Option<&str>, iterable: Value, body: Vec<Value>) -> Value {
    let mut obj = json!({
        "type": "Stmt.For", "item": item, "iterable": iterable,
        "body": body, "trim": "none"
    });
    if let Some(i) = index { obj["index"] = json!(i); }
    obj
}

fn stmt_block(name: &str, body: Vec<Value>) -> Value {
    json!({ "type": "Stmt.Block", "name": name, "body": body, "trim": "none" })
}

fn stmt_extends(parent: Value) -> Value {
    json!({ "type": "Stmt.Extends", "parent": parent, "trim": "none" })
}

fn stmt_include(path: Value) -> Value {
    json!({ "type": "Stmt.Include", "path": path, "trim": "none" })
}

// ---------------------------------------------------------------------------
// Basic Text / Interpolation
// ---------------------------------------------------------------------------

#[test]
fn text_only_emits_push() {
    let ir = doc(vec![text("Hello, "), text("world!")]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("__out.push(\"Hello, \");"));
    assert!(r.contains("__out.push(\"world!\");"));
    assert!(r.contains("return __out.join(\"\");"));
    assert!(res.diagnostics.is_empty());
}

#[test]
fn comment_is_skipped() {
    let ir = doc(vec![text("a"), comment("drop me"), text("b")]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("__out.push(\"a\");"));
    assert!(r.contains("__out.push(\"b\");"));
    assert!(!r.contains("drop me"));
    assert!(res.diagnostics.is_empty());
}

#[test]
fn interpolation_plain_emits_html_escape() {
    let ir = doc(vec![interp(ident("name"), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round_body(&res.code);
    assert!(r.contains("let __v = ctx.name;"));
    assert!(r.contains("__out.push(htmlEscape(valueToString(__v)));"));
}

#[test]
fn interpolation_raw_flag_skips_html_escape() {
    let ir = doc(vec![interp(ident("name"), true)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round_body(&res.code);
    assert!(r.contains("let __v = ctx.name;"));
    assert!(r.contains("__out.push(valueToString(__v));"));
    assert!(!r.contains("htmlEscape"));
}

#[test]
fn interpolation_pipe_escape_skips_html_escape() {
    let ir = doc(vec![interp(pipe("escape", ident("name"), vec![]), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round_body(&res.code);
    assert!(r.contains("applyFilter(\"escape\""));
    assert!(r.contains("__out.push(valueToString(__v));"));
    assert!(!r.contains("htmlEscape"));
}

#[test]
fn interpolation_pipe_raw_skips_html_escape() {
    let ir = doc(vec![interp(pipe("raw", ident("name"), vec![]), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round_body(&res.code);
    assert!(r.contains("__out.push(valueToString(__v));"));
    assert!(!r.contains("htmlEscape"));
}

#[test]
fn interpolation_pipe_safe_skips_html_escape() {
    let ir = doc(vec![interp(pipe("safe", ident("name"), vec![]), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round_body(&res.code);
    assert!(!r.contains("htmlEscape"));
}

#[test]
fn interpolation_pipe_uppercase_still_escapes() {
    let ir = doc(vec![interp(pipe("uppercase", ident("name"), vec![]), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round_body(&res.code);
    assert!(r.contains("htmlEscape(valueToString(__v))"));
}

// ---------------------------------------------------------------------------
// If / ElseIf / Else
// ---------------------------------------------------------------------------

#[test]
fn if_statement_emits_is_truthy() {
    let ir = doc(vec![stmt_if(
        ident("ok"),
        vec![text("yes")],
        vec![],
        Some(vec![text("no")]),
    )]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("if (isTruthy(ctx.ok)) {"));
    assert!(r.contains("__out.push(\"yes\");"));
    assert!(r.contains("} else {"));
    assert!(r.contains("__out.push(\"no\");"));
}

#[test]
fn if_elseif_chain() {
    let ir = doc(vec![stmt_if(
        ident("a"),
        vec![text("a")],
        vec![
            stmt_elseif(ident("b"), vec![text("b")]),
            stmt_elseif(ident("c"), vec![text("c")]),
        ],
        Some(vec![text("fallback")]),
    )]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("if (isTruthy(ctx.a)) {"));
    assert!(r.contains("else if (isTruthy(ctx.b)) {"));
    assert!(r.contains("else if (isTruthy(ctx.c)) {"));
    assert!(r.contains("} else {"));
}

// ---------------------------------------------------------------------------
// For
// ---------------------------------------------------------------------------

#[test]
fn for_with_index() {
    let ir = doc(vec![stmt_for(
        "item",
        Some("i"),
        ident("items"),
        vec![text("#"), interp(ident("item"), false)],
    )]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("let __v = ctx.items;"));
    assert!(r.contains("for (let __i = 0; __i < __v.length; __i++) {"));
    assert!(r.contains("ctx[\"item\"] = __v[__i];"));
    assert!(r.contains("ctx[\"i\"] = __i;"));
    // Loop variables are scoped and restored after the loop.
    assert!(r.contains("Object.prototype.hasOwnProperty.call(ctx, \"item\")"));
    assert!(r.contains("if (__v_item_has) ctx[\"item\"] = __v_item_prev; else delete ctx[\"item\"];"));
}

#[test]
fn for_without_index_skips_ctx_index_assignment() {
    let ir = doc(vec![stmt_for(
        "x", None, ident("items"), vec![interp(ident("x"), false)],
    )]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(!r.contains("ctx[\"i\"]"));
    assert!(r.contains("ctx[\"x\"] = __v[__i];"));
    assert!(!r.contains("_index_has"));
}

#[test]
fn two_loops_in_one_template_get_distinct_scopes() {
    let ir = doc(vec![
        stmt_for("a", None, ident("xs"), vec![text("x")]),
        stmt_for("b", Some("i"), ident("ys"), vec![text("y")]),
    ]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    // Each loop uses its own iterable temp and block scope.
    assert!(r.contains("let __v = ctx.xs;"));
    assert!(r.contains("let __v1 = ctx.ys;"));
    assert!(!r.contains("let __it"));
}

// ---------------------------------------------------------------------------
// Expression shapes
// ---------------------------------------------------------------------------

#[test]
fn expr_literal_number_string() {
    let ir = doc(vec![interp(lit(json!(42)), false), text(" "),
                       interp(lit(json!("hi")), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("let __v = 42;"));
    assert!(r.contains("let __v1 = \"hi\";"));
}

#[test]
fn expr_literal_null_bool() {
    let ir = doc(vec![interp(lit(json!(null)), false), text(" "),
                       interp(lit(json!(true)), false), text(" "),
                       interp(lit(json!(false)), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("null"));
    assert!(r.contains("true"));
    assert!(r.contains("false"));
}

#[test]
fn expr_literal_object_array() {
    let ir = doc(vec![interp(lit(json!({ "a": [1, 2], "b": null })), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("\"a\""));
    assert!(r.contains("\"b\""));
    assert!(r.contains("[1, 2]"));
}

#[test]
fn expr_reserved_word_uses_bracket() {
    let ir = doc(vec![interp(ident("if"), false), text(" "),
                       interp(ident("for"), false), text(" "),
                       interp(ident("await"), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("ctx[\"if\"]"));
    assert!(r.contains("ctx[\"for\"]"));
    assert!(r.contains("ctx[\"await\"]"));
}

#[test]
fn expr_member_emits_helper() {
    let ir = doc(vec![interp(member(ident("user"), "name"), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("exprMember(ctx.user, \"name\")"));
}

#[test]
fn expr_index_emits_helper() {
    let ir = doc(vec![interp(index(ident("arr"), lit(json!(0))), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("exprIndex(ctx.arr, 0)"));
}

#[test]
fn expr_binary_emits_helper() {
    let ir = doc(vec![interp(binary("+", lit(json!(1)), lit(json!(2))), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("binary(\"+\", 1, 2)"));
}

#[test]
fn expr_unary_emits_helper() {
    let ir = doc(vec![interp(unary("-", ident("x")), false), text(" "),
                       interp(unary("!", ident("y")), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("unary(\"-\", ctx.x)"));
    assert!(r.contains("unary(\"!\", ctx.y)"));
}

// ---------------------------------------------------------------------------
// ExprCall / ExprTest — closed namespaces
// ---------------------------------------------------------------------------

#[test]
fn expr_call_unknown_function_is_rejected() {
    let ir = doc(vec![interp(call(ident("f"), vec![lit(json!(1))]), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    assert!(!res.diagnostics.is_empty());
    assert_eq!(res.diagnostics[0].code, "dejavu.template.unknown-function");
    assert!(res.diagnostics[0].severity == dejavu_diagnostic::Severity::Error);
    assert!(res.code.contains("export function render")); // module still parses
}

#[test]
fn expr_call_member_callee_stays_closed() {
    let ir = doc(vec![interp(
        call(member(ident("user"), "format"), vec![]),
        false,
    )]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    assert_eq!(res.diagnostics[0].code, "dejavu.template.calls-not-supported");
}

#[test]
fn expr_call_builtin_emits_call_function() {
    let ir = doc(vec![interp(
        call(ident("range"), vec![lit(json!(0)), lit(json!(3))]),
        false,
    )]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round_body(&res.code);
    assert!(r.contains("callFunction(\"range\", [0, 3])"), "{r}");
    assert!(res.diagnostics.is_empty(), "{:?}", res.diagnostics);
}

#[test]
fn expr_call_arity_violation_is_diagnostic() {
    let ir = doc(vec![interp(call(ident("range"), vec![lit(json!(0))]), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    assert_eq!(res.diagnostics[0].code, "dejavu.template.function-arity");
}

#[test]
fn expr_test_emits_apply_test() {
    let ir = doc(vec![interp(test_node(ident("n"), "even", vec![], false), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round_body(&res.code);
    assert!(r.contains("applyTest(\"even\", ctx.n, [], false)"), "{r}");
    assert!(res.diagnostics.is_empty(), "{:?}", res.diagnostics);
}

#[test]
fn expr_test_negation_and_alias_use_canonical_name() {
    let ir = doc(vec![interp(test_node(ident("v"), "none", vec![], true), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round_body(&res.code);
    assert!(r.contains("applyTest(\"null\", ctx.v, [], true)"), "{r}");
}

#[test]
fn expr_test_unknown_name_is_diagnostic() {
    let ir = doc(vec![interp(test_node(ident("v"), "nope", vec![], false), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    assert_eq!(res.diagnostics[0].code, "dejavu.template.unknown-test");
}

#[test]
fn expr_test_arity_violation_is_diagnostic() {
    let ir = doc(vec![interp(
        test_node(ident("v"), "empty", vec![lit(json!(1))], false),
        false,
    )]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    assert_eq!(res.diagnostics[0].code, "dejavu.template.test-arity");
}

// ---------------------------------------------------------------------------
// Unsupported nodes — StmtBlock / StmtExtends / StmtInclude
// ---------------------------------------------------------------------------

#[test]
fn stmt_block_emits_throw_and_diagnostic() {
    let ir = doc(vec![stmt_block("head", vec![text("x")])]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains(
        "throw new Error(\"dejavu.compile.not-supported-in-single-template: Stmt.Block (head)\")"
    ));
    assert_eq!(res.diagnostics.len(), 1);
    assert!(res.diagnostics[0].message.as_ref().unwrap().contains("Stmt.Block"));
}

#[test]
fn stmt_extends_emits_throw_and_diagnostic() {
    let ir = doc(vec![stmt_extends(lit(json!("layout.dejavu")))]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("Stmt.Extends"));
}

#[test]
fn stmt_include_emits_throw_and_diagnostic() {
    let ir = doc(vec![stmt_include(lit(json!("partials/header.dejavu")))]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("Stmt.Include"));
}

// ---------------------------------------------------------------------------
// Strict undefined mode
// ---------------------------------------------------------------------------

#[test]
fn strict_undefined_injects_check_after_identifier_interp() {
    let ir = doc(vec![interp(ident("missing"), false)]);
    let res = compile_opts(
        &serde_json::to_string(&ir).unwrap(),
        r#"{"strictUndefined": true}"#,
    );
    let r = round(&res.code);
    assert!(r.contains(
        "if (__strict && __v === undefined) throw new Error(\"dejavu.template.undefined-variable: missing\");"
    ));
}

#[test]
fn non_strict_defaults_runtime_strict_off() {
    let ir = doc(vec![interp(ident("missing"), false)]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    // The check is emitted but gated by `__strict`, whose build-time default
    // is false here.
    assert!(res
        .code
        .contains("const __strict = options.strictUndefined === undefined ? false : options.strictUndefined === true;"));
    assert!(res.code.contains("if (__strict && __v === undefined)"));
}

// ---------------------------------------------------------------------------
// Module structure
// ---------------------------------------------------------------------------

#[test]
fn module_has_runtime_import_and_render_export() {
    let ir = doc(vec![]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    let r = round(&res.code);
    assert!(r.contains("from \"@doki-land/dejavu-engine/runtime\""));
    assert!(r.contains("export function render(ctx, options = {})"));
    assert!(r.contains("if (!ctx || typeof ctx !== 'object') ctx = {};"));
    assert!(r.contains("const __out = [];"));
    assert!(r.contains("return __out.join(\"\");"));
}

#[test]
fn entry_id_default_is_entry() {
    let ir = doc(vec![]);
    let res = compile(&serde_json::to_string(&ir).unwrap());
    assert_eq!(res.entry_id, "__entry__");
    assert!(res.dependencies.is_empty());
}

#[test]
fn entry_id_from_options() {
    let ir = doc(vec![]);
    let res = compile_opts(&serde_json::to_string(&ir).unwrap(),
                           r#"{"entryId":"theme:layout.dejavu"}"#);
    assert_eq!(res.entry_id, "theme:layout.dejavu");
}

// ---------------------------------------------------------------------------
// Line origins (source map input)
// ---------------------------------------------------------------------------

/// Attach a template span to a fixture node.
fn spanned(mut node: Value, start: u32, end: u32) -> Value {
    node["span"] = json!({ "start": start, "end": end });
    node
}

#[test]
fn origins_point_each_construct_at_its_template_span() {
    let ir = doc(vec![
        spanned(text("hello"), 0, 5),
        spanned(interp(ident("name"), false), 5, 15),
        spanned(
            stmt_if(ident("ok"), vec![spanned(text("yes"), 21, 24)], vec![], None),
            15,
            40,
        ),
    ]);
    let res = compile(&serde_json::to_string(&ir).unwrap());

    // Module header is 7 lines when there are no template imports.
    assert_eq!(
        res.origins,
        vec![
            LineOrigin { line: 7, start: 0, end: 5 },
            LineOrigin { line: 8, start: 5, end: 15 },
            // The interpolation emits its (runtime-gated) strict check, so the
            // following `if` starts one body line later.
            LineOrigin { line: 11, start: 15, end: 40 },
            LineOrigin { line: 12, start: 21, end: 24 },
        ]
    );

    // Every origin must land on a real module line.
    let total = res.code.lines().count() as u32;
    for origin in &res.origins {
        assert!(origin.line < total, "origin {origin:?} outside {total} lines");
        assert!(origin.start <= origin.end);
    }
}

#[test]
fn origins_skip_comments_and_spans_absent_from_fixtures() {
    let res = compile(&serde_json::to_string(&doc(vec![text("a"), comment("b")])).unwrap());
    assert!(res.origins.is_empty());
}

// ---------------------------------------------------------------------------
// Round-trip with Node stub (best-effort, skipped if node unavailable)
// ---------------------------------------------------------------------------

#[test]
fn generated_code_runs_in_node_stub() {
    let ir = doc(vec![
        text("Hello, "),
        interp(pipe("escape", ident("name"), vec![]), false),
        text("! items=["),
        stmt_for("item", Some("i"), ident("items"), vec![
            interp(ident("i"), false),
            text(":"),
            interp(ident("item"), false),
            text(" "),
        ]),
        text("] count="),
        interp(binary("+", ident("count"), lit(json!(1))), false),
    ]);
    let res = compile(&serde_json::to_string(&ir).unwrap());

    let node_stub = r#"
const __rt = {
  htmlEscape: (s) => String(s).replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c])),
  valueToString: (v) => (v == null) ? '' : String(v),
  isTruthy: (v) => v !== null && v !== undefined && v !== '' && v !== 0 && v !== false,
  applyFilter: (name, v, args) => {
    switch (name) {
      case 'escape': case 'e': return String(v).replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));
      case 'raw': case 'safe': return v;
      case 'uppercase': case 'upper': return String(v).toUpperCase();
      default: return v;
    }
  },
  binary: (op, l, r) => {
    switch (op) {
      case '+': return l + r;
      case '-': return l - r;
      case '*': return l * r;
      case '/': return l / r;
      case '==': return l == r;
      case '!=': return l != r;
      case '<': return l < r;
      case '<=': return l <= r;
      case '>': return l > r;
      case '>=': return l >= r;
      case '&&': return l && r;
      case '||': return l || r;
      case 'in': return r != null && l in r;
      default: return null;
    }
  },
  unary: (op, a) => {
    switch (op) { case '!': return !a; case '-': return -a; case '+': return +a; default: return null; }
  },
  exprMember: (obj, prop) => obj == null ? null : obj[prop] ?? null,
  exprIndex: (arr, i) => arr == null ? null : arr[i] ?? null,
};
const { htmlEscape, applyFilter, isTruthy, binary, unary, valueToString, exprMember, exprIndex } = __rt;
"#;

    // Strip the ESM import/export lines so this runs as a plain script.
    let mut js_lines: Vec<&str> = res.code.lines().collect();
    js_lines.retain(|l| !l.trim().starts_with("import "));
    let body = js_lines.join("\n").replace("export function render", "function render");

    let full = format!(
        "{node_stub}\n{body}\nconsole.log(render({{ name: '<world>', items: ['a','b'], count: 10 }}, {{}}));"
    );

    match std::process::Command::new("node").args(["-e", &full]).output() {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            assert_eq!(stdout.trim(), "Hello, &lt;world&gt;! items=[0:a 1:b ] count=11");
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            panic!("node failed: {stderr}");
        }
        Err(_) => {
            eprintln!("SKIP: node not available, skipping runtime test");
        }
    }
}
