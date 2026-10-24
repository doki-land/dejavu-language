//! AOT loop scoping and build-time `strictUndefined` tests.
//!
//! These cover two codegen semantic defects:
//!
//! 1. `Stmt.For` used to declare a shared `let __it` in the function body
//!    (two loops → SyntaxError) and wrote loop variables into `ctx` without
//!    restoring them afterwards (leaking into the caller context and nested
//!    includes).
//! 2. `build_module` ignored `CodegenOptions.strict_undefined`, hard-coding
//!    the runtime `__strict` flag to the per-call option only.
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

fn interp(expression: Value, raw: bool) -> Value {
    json!({ "type": "Interpolation", "expression": expression, "trim": "none", "raw": raw })
}

fn ident(name: &str) -> Value {
    json!({ "type": "Expr.Identifier", "name": name })
}

fn stmt_for(item: &str, index: Option<&str>, iterable: Value, body: Vec<Value>) -> Value {
    let mut obj = json!({
        "type": "Stmt.For", "item": item, "iterable": iterable,
        "body": body, "trim": "none"
    });
    if let Some(i) = index {
        obj["index"] = json!(i);
    }
    obj
}

fn compile(ir: &Value, opts_json: Option<&str>) -> String {
    precompile_single(&serde_json::to_string(ir).unwrap(), opts_json)
        .expect("compile should succeed")
        .code
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
  exprMember: (o, p) => o == null ? null : (o[p] === undefined ? null : o[p]),
  exprIndex: (o, i) => o == null ? null : (o[i] === undefined ? null : o[i]),
  runtimeError: (code, detail) => {
    const err = new Error(detail ? `${code}: ${detail}` : code);
    err.code = code;
    err.stage = 'render';
    return err;
  },
};
const { htmlEscape, applyFilter, isTruthy, binary, unary, valueToString, exprMember, exprIndex, runtimeError } = __rt;
"#;

/// `Some(Ok((rendered, probe)))`, `Some(Err(message))`, or `None` when Node is
/// not installed. `probe_js` is evaluated against `__ctx` *after* rendering.
fn run_render(
    code: &str,
    ctx_js: &str,
    opts_js: &str,
    probe_js: &str,
) -> Option<Result<(String, Value), String>> {
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
            let parsed: Value = serde_json::from_str(stdout.trim())
                .unwrap_or_else(|e| panic!("bad node output {stdout:?}: {e}"));
            if let Some(err) = parsed["error"].as_str() {
                Some(Err(err.to_string()))
            } else {
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
        stmt_for("x", None, ident("xs"), vec![interp(ident("x"), false), text(",")]),
        text("]B["),
        stmt_for(
            "y",
            Some("k"),
            ident("ys"),
            vec![
                interp(ident("k"), false),
                text("="),
                interp(ident("y"), false),
                text(","),
            ],
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
        stmt_for("item", None, ident("items"), vec![interp(ident("item"), false)]),
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
    if let Some(out) = run_render(
        &code,
        "({ items: ['a','b'] })",
        "{}",
        "Object.prototype.hasOwnProperty.call(__ctx, 'item')",
    ) {
        let (rendered, probe) = out.unwrap();
        assert_eq!(rendered, "[ab]");
        assert_eq!(probe, json!(false));
    }
}

#[test]
fn nested_loops_do_not_clobber_each_other() {
    let ir = doc(vec![stmt_for(
        "outer",
        None,
        ident("rows"),
        vec![
            text("("),
            stmt_for(
                "inner",
                Some("j"),
                ident("outer"),
                vec![
                    interp(ident("j"), false),
                    text(":"),
                    interp(ident("inner"), false),
                    text(","),
                ],
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
        r#"({ outer: Object.prototype.hasOwnProperty.call(__ctx, 'outer'), inner: Object.prototype.hasOwnProperty.call(__ctx, 'inner'), j: Object.prototype.hasOwnProperty.call(__ctx, 'j') })"#,
    ) {
        let (rendered, probe) = out.unwrap();
        assert_eq!(rendered, "(0:a,1:b,=>a,b)(0:c,=>c)");
        assert_eq!(probe["outer"], json!(false));
        assert_eq!(probe["inner"], json!(false));
        assert_eq!(probe["j"], json!(false));
    }
}

#[test]
fn nested_loops_with_same_item_name_restore_outer_value() {
    let ir = doc(vec![stmt_for(
        "v",
        None,
        ident("rows"),
        vec![
            stmt_for("v", None, ident("v"), vec![interp(ident("v"), false)]),
            text("|"),
            interp(ident("v"), false),
        ],
    )]);
    let code = compile(&ir, None);

    if let Some(out) = run_render(&code, "({ rows: [['a','b']] })", "{}", "null") {
        assert_eq!(out.unwrap().0, "ab|a,b");
    }
}

// ---------------------------------------------------------------------------
// Defect 2: build-time strictUndefined + per-call override
// ---------------------------------------------------------------------------

#[test]
fn strict_undefined_build_flag_applies_when_call_omits_option() {
    let ir = doc(vec![interp(ident("missing"), false)]);
    let code = compile(&ir, Some(r#"{"strictUndefined": true}"#));

    assert!(code.contains(
        "const __strict = options.strictUndefined === undefined ? true : options.strictUndefined === true;"
    ));
    if let Some(out) = run_render(&code, "({})", "{}", "null") {
        let err = out.expect_err("missing variable should throw");
        assert!(err.contains("dejavu.template.undefined-variable: missing"), "{err}");
    }
}

#[test]
fn call_time_strict_true_overrides_build_time_false() {
    let ir = doc(vec![interp(ident("missing"), false)]);
    let code = compile(&ir, None);

    if let Some(out) = run_render(&code, "({})", "{ strictUndefined: true }", "null") {
        let err = out.expect_err("call-time strict should throw");
        assert!(err.contains("dejavu.template.undefined-variable: missing"), "{err}");
    }
}

#[test]
fn call_time_strict_false_overrides_build_time_true() {
    let ir = doc(vec![interp(ident("missing"), false)]);
    let code = compile(&ir, Some(r#"{"strictUndefined": true}"#));

    if let Some(out) = run_render(&code, "({})", "{ strictUndefined: false }", "null") {
        let (rendered, _) = out.unwrap();
        assert_eq!(rendered, "");
    }
}

#[test]
fn build_time_strict_false_and_call_time_default_render_empty() {
    let ir = doc(vec![interp(ident("missing"), false)]);
    let code = compile(&ir, None);

    if let Some(out) = run_render(&code, "({})", "{}", "null") {
        assert_eq!(out.unwrap().0, "");
    }
}
