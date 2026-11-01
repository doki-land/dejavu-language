//! AOT loop scoping and render-time `strictUndefined` tests.
//!
//! These cover two codegen semantic defects:
//!
//! 1. `Stmt.For` used to declare a shared `let __it` in the function body
//!    (two loops → SyntaxError) and wrote loop variables into `ctx` without
//!    restoring them afterwards (leaking into the caller context and nested
//!    includes).
//! 2. `build_module` used to bake `CodegenOptions.strict_undefined` into the
//!    generated `__strict` default. Strict is a render-time option only: the
//!    module reads `options.strictUndefined` per call and never freezes a
//!    build-time value (same option as JIT / WASM).
//!
//! Generated modules are executed under a small Node.js runtime stub (same
//! pattern as `single_template.rs`); if Node is unavailable the runtime half
//! of each test is skipped.

use dejavu_codegen::precompile_single;
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// IR fixture builders
// ---------------------------------------------------------------------------

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

fn interp(expression: Value, raw: bool) -> Value {
    json!({ "type": "Interpolation", "expression": expression, "trim": "none", "raw": raw })
}

fn ident(name: &str) -> Value {
    json!({ "type": "Expr.Identifier", "name": name })
}

fn stmt_for(item: &str, iterable: Value, body: Vec<Value>) -> Value {
    json!({
        "type": "Stmt.For", "item": item, "iterable": iterable,
        "body": body, "trim": "none"
    })
}

fn member(object: Value, property: &str) -> Value {
    json!({ "type": "Expr.Member", "object": object, "property": property })
}

fn lit(value: Value) -> Value {
    json!({ "type": "Expr.Literal", "value": value })
}

fn index(object: Value, idx: Value) -> Value {
    json!({ "type": "Expr.Index", "object": object, "index": idx })
}

fn compile(ir: &Value, opts_json: Option<&str>) -> String {
    precompile_single(&serde_json::to_string(ir).unwrap(), opts_json).expect("compile should succeed").code
}

// ---------------------------------------------------------------------------
// Node runtime stub + runner
// ---------------------------------------------------------------------------

const NODE_STUB: &str = r#"
const __rt = {
  htmlEscape: (s) => String(s).replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c])),
  valueToString: (v) => (v == null) ? '' : String(v),
  isTruthy: (v) => v !== null && v !== undefined && v !== '' && v !== 0 && v !== false,
  applyFilter: (name, v) => v,
  binary: (op, l, r) => (op === '+' ? (typeof l === 'string' || typeof r === 'string' ? String(l) + String(r) : l + r) : null),
  unary: (op, a) => (op === '!' ? !a : null),
  exprIdent: (ctx, name, strict) => {
    if (Object.prototype.hasOwnProperty.call(ctx, name)) return ctx[name];
    if (strict) throw runtimeError("dejavu.template.undefined-variable", name, { name });
    return null;
  },
  exprMember: (o, p, strict, path) => {
    if (o == null) return null;
    if (strict && typeof o === 'object' && !Array.isArray(o) && !Object.prototype.hasOwnProperty.call(o, p)) {
      throw runtimeError("dejavu.template.undefined-member", p, { name: p, path });
    }
    const v = o[p];
    return v === undefined ? null : v;
  },
  exprIndex: (o, i, strict, path) => {
    if (o == null) return null;
    if (Array.isArray(o)) {
      if (strict && typeof i === 'number' && Number.isInteger(i) && i >= 0 && i >= o.length) {
        throw runtimeError("dejavu.template.undefined-index", String(i), { name: String(i), path });
      }
      const v = o[Number(i) | 0];
      return v === undefined ? null : v;
    }
    if (typeof o === 'object') {
      if (strict && typeof i === 'string' && !Object.prototype.hasOwnProperty.call(o, i)) {
        throw runtimeError("dejavu.template.undefined-index", i, { name: i, path });
      }
      const v = o[String(i)];
      return v === undefined ? null : v;
    }
    return null;
  },
  runtimeError: (code, detail) => {
    const err = new Error(detail ? `${code}: ${detail}` : code);
    err.code = code;
    err.stage = 'render';
    return err;
  },
};
const { htmlEscape, applyFilter, isTruthy, binary, unary, valueToString, exprIdent, exprMember, exprIndex, runtimeError } = __rt;
"#;

/// `Some(Ok((rendered, probe)))`, `Some(Err(message))`, or `None` when Node is
/// not installed. `probe_js` is evaluated against `__ctx` *after* rendering.
fn run_render(code: &str, ctx_js: &str, opts_js: &str, probe_js: &str) -> Option<Result<(String, Value), String>> {
    let body = code
        .lines()
        .filter(|l| !l.trim_start().starts_with("import ") && !l.trim_start().starts_with("export const "))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("export function render", "function render");

    let script = format!(
        "{NODE_STUB}\n{body}\n\
const __ctx = {ctx_js};\n\
let __value, __error;\n\
try {{ __value = render(__ctx, {opts_js}); }} catch (e) {{ __error = String(e && e.message ? e.message : e); }}\n\
let __probe;\n\
try {{ __probe = ({probe_js}); }} catch (e) {{ __probe = null; }}\n\
console.log(JSON.stringify({{ value: __value, error: __error, probe: __probe }}));\n"
    );

    match std::process::Command::new("node").args(["-e", &script]).output() {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let parsed: Value =
                serde_json::from_str(stdout.trim()).unwrap_or_else(|e| panic!("bad node output {stdout:?}: {e}"));
            if let Some(err) = parsed["error"].as_str() {
                Some(Err(err.to_string()))
            }
            else {
                let value = parsed["value"].as_str().unwrap_or("").to_string();
                Some(Ok((value, parsed["probe"].clone())))
            }
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            panic!("node failed: {stderr}");
        }
        Err(_) => {
            eprintln!("SKIP: node not available, skipping runtime assertions");
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Defect 1: loop scoping / context restoration
// ---------------------------------------------------------------------------

#[test]
fn two_loops_in_one_template_load_and_render() {
    let ir = doc(vec![
        text("A["),
        stmt_for("x", ident("xs"), vec![interp(ident("x"), false), text(",")]),
        text("]B["),
        stmt_for(
            "y",
            ident("ys"),
            vec![interp(member(ident("forloop"), "index0"), false), text("="), interp(ident("y"), false), text(",")],
        ),
        text("]"),
    ]);
    let code = compile(&ir, None);

    // The old emitter produced two `let __it` declarations → SyntaxError.
    if let Some(out) = run_render(&code, "({ xs: ['1','2'], ys: ['u','v'] })", "{}", "null") {
        assert_eq!(out.unwrap().0, "A[1,2,]B[0=u,1=v,]");
    }
}

#[test]
fn loop_item_is_restored_after_loop() {
    let ir = doc(vec![
        text("["),
        stmt_for("item", ident("items"), vec![interp(ident("item"), false)]),
        text("]"),
        interp(ident("item"), false),
    ]);
    let code = compile(&ir, None);

    // Existing `item` in the caller context survives the loop unchanged.
    if let Some(out) = run_render(
        &code,
        "({ items: ['a','b'], item: 'orig' })",
        "{}",
        r#"({ own: Object.prototype.hasOwnProperty.call(__ctx, 'item'), value: __ctx.item })"#,
    ) {
        let (rendered, probe) = out.unwrap();
        assert_eq!(rendered, "[ab]orig");
        assert_eq!(probe["own"], json!(true));
        assert_eq!(probe["value"], json!("orig"));
    }

    // Without a pre-existing `item`, the loop must delete the key instead of
    // leaving the last iteration value behind.
    if let Some(out) = run_render(&code, "({ items: ['a','b'] })", "{}", "Object.prototype.hasOwnProperty.call(__ctx, 'item')")
    {
        let (rendered, probe) = out.unwrap();
        assert_eq!(rendered, "[ab]");
        assert_eq!(probe, json!(false));
    }
}

#[test]
fn nested_loops_do_not_clobber_each_other() {
    let ir = doc(vec![stmt_for(
        "outer",
        ident("rows"),
        vec![
            text("("),
            stmt_for(
                "inner",
                ident("outer"),
                vec![interp(member(ident("forloop"), "index0"), false), text(":"), interp(ident("inner"), false), text(",")],
            ),
            text("=>"),
            // After the inner loop, `outer` must hold the current row again.
            interp(ident("outer"), false),
            text(")"),
        ],
    )]);
    let code = compile(&ir, None);

    if let Some(out) = run_render(
        &code,
        "({ rows: [['a','b'], ['c']] })",
        "{}",
        r#"({ outer: Object.prototype.hasOwnProperty.call(__ctx, 'outer'), inner: Object.prototype.hasOwnProperty.call(__ctx, 'inner'), forloop: Object.prototype.hasOwnProperty.call(__ctx, 'forloop') })"#,
    ) {
        let (rendered, probe) = out.unwrap();
        assert_eq!(rendered, "(0:a,1:b,=>a,b)(0:c,=>c)");
        assert_eq!(probe["outer"], json!(false));
        assert_eq!(probe["inner"], json!(false));
        assert_eq!(probe["forloop"], json!(false));
    }
}

#[test]
fn forloop_nesting_depth_and_restore() {
    let ir = doc(vec![stmt_for(
        "row",
        ident("rows"),
        vec![
            text("["),
            interp(member(ident("forloop"), "depth"), false),
            stmt_for(
                "cell",
                ident("row"),
                vec![
                    interp(member(ident("forloop"), "depth"), false),
                    text(":"),
                    interp(member(ident("forloop"), "index"), false),
                ],
            ),
            // After the inner loop the outer `forloop` is visible again.
            interp(member(ident("forloop"), "depth"), false),
            text("]"),
        ],
    )]);
    let code = compile(&ir, None);

    if let Some(out) =
        run_render(&code, "({ rows: [['a','b'], ['c']] })", "{}", "Object.prototype.hasOwnProperty.call(__ctx, 'forloop')")
    {
        let (rendered, probe) = out.unwrap();
        assert_eq!(rendered, "[12:12:21][12:11]");
        assert_eq!(probe, json!(false));
    }
}

#[test]
fn forloop_shadows_and_restores_context_value() {
    let ir = doc(vec![
        stmt_for("x", ident("xs"), vec![interp(member(ident("forloop"), "index"), false)]),
        text("|"),
        interp(ident("forloop"), false),
    ]);
    let code = compile(&ir, None);

    if let Some(out) = run_render(&code, "({ xs: ['a','b'], forloop: 'ctx' })", "{}", "null") {
        assert_eq!(out.unwrap().0, "12|ctx");
    }
}

#[test]
fn nested_loops_with_same_item_name_restore_outer_value() {
    let ir = doc(vec![stmt_for(
        "v",
        ident("rows"),
        vec![stmt_for("v", ident("v"), vec![interp(ident("v"), false)]), text("|"), interp(ident("v"), false)],
    )]);
    let code = compile(&ir, None);

    if let Some(out) = run_render(&code, "({ rows: [['a','b']] })", "{}", "null") {
        assert_eq!(out.unwrap().0, "ab|a,b");
    }
}

// ---------------------------------------------------------------------------
// Defect 2 (fixed): strict is a render-time option only — codegen never bakes
// a default in, and the per-call option is the single switch.
// ---------------------------------------------------------------------------

#[test]
fn build_time_strict_flag_is_ignored_and_call_default_is_lenient() {
    let ir = doc(vec![interp(ident("missing"), false)]);
    let code = compile(&ir, Some(r#"{"strictUndefined": true}"#));

    // The generated module reads the caller's option verbatim; a build-time
    // `strictUndefined` must not appear anywhere in the emitted default.
    assert!(code.contains("const __strict = options.strictUndefined === true;"));
    assert!(!code.contains("=== undefined"));
    if let Some(out) = run_render(&code, "({})", "{}", "null") {
        assert_eq!(out.unwrap().0, "", "call without strict option stays lenient");
    }
}

#[test]
fn call_time_strict_true_throws_undefined_variable() {
    let ir = doc(vec![interp(ident("missing"), false)]);
    let code = compile(&ir, None);

    if let Some(out) = run_render(&code, "({})", "{ strictUndefined: true }", "null") {
        let err = out.expect_err("call-time strict should throw");
        assert!(err.contains("dejavu.template.undefined-variable: missing"), "{err}");
    }
}

#[test]
fn call_time_strict_false_renders_lenient() {
    let ir = doc(vec![interp(ident("missing"), false)]);
    let code = compile(&ir, None);

    if let Some(out) = run_render(&code, "({})", "{ strictUndefined: false }", "null") {
        let (rendered, _) = out.unwrap();
        assert_eq!(rendered, "");
    }
}

#[test]
fn call_time_strict_true_flags_missing_member_and_index() {
    let ir = doc(vec![
        interp(member(ident("user"), "nickname"), false),
        text("|"),
        interp(index(ident("items"), lit(json!(3))), false),
    ]);
    let code = compile(&ir, None);

    if let Some(out) = run_render(&code, "({ user: { name: 'a' }, items: ['x'] })", "{ strictUndefined: true }", "null") {
        let err = out.expect_err("strict member access should throw");
        assert!(err.contains("dejavu.template.undefined-member: nickname"), "{err}");
    }

    if let Some(out) = run_render(&code, "({ user: { nickname: 'n' }, items: ['x'] })", "{ strictUndefined: true }", "null") {
        let err = out.expect_err("strict index access should throw");
        assert!(err.contains("dejavu.template.undefined-index: 3"), "{err}");
    }

    // Lenient default: both missing accesses collapse to empty segments.
    if let Some(out) = run_render(&code, "({ user: { name: 'a' }, items: ['x'] })", "{}", "null") {
        assert_eq!(out.unwrap().0, "|");
    }
}
