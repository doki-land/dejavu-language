//! End-to-end catalog AOT codegen tests.
//!
//! Strategy (mirrors `single_template.rs`):
//! 1. Hand-write IR JSON fixtures for every template.
//! 2. Assemble the catalog JSON wire format.
//! 3. Call [`precompile_catalog`] with an explicit `entryId` option.
//! 4. Assert structural facts.
//! 5. Run generated JS under a Node.js stub when node is available.

use dejavu_codegen::precompile_catalog;
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn round(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn compile_catalog(files: &[(&str, Value)], entry_path: &str) -> dejavu_codegen::PrecompileCatalogResult {
    let entry_id = format!("main:{entry_path}");
    let opts = format!(r#"{{"entryId":"{entry_id}"}}"#);
    let file_map = files.iter().map(|(path, doc)| {
        (path.to_string(), Value::String(serde_json::to_string(doc).unwrap()))
    }).collect::<serde_json::Map<String, Value>>();
    let root = json!({
        "name": "main",
        "scheme": "main",
        "priority": 10,
        "files": Value::Object(file_map),
    });
    let catalog_json = serde_json::to_string(&json!({
        "roots": [root],
        "extensions": [".dejavu", ".yarte", ""],
    }))
    .unwrap();
    precompile_catalog("", &catalog_json, Some(&opts)).expect("catalog compile should succeed")
}

fn errors(r: &dejavu_codegen::PrecompileCatalogResult) -> Vec<&dejavu_diagnostic::Diagnostic> {
    r.diagnostics.iter().filter(|d| d.severity == dejavu_diagnostic::Severity::Error).collect()
}

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
fn text(v: &str) -> Value { json!({ "type": "Text", "value": v }) }
fn interp(e: Value, raw: bool) -> Value {
    json!({ "type": "Interpolation", "expression": e, "trim": "none", "raw": raw })
}
fn ident(n: &str) -> Value { json!({ "type": "Expr.Identifier", "name": n }) }
fn lit(v: Value) -> Value { json!({ "type": "Expr.Literal", "value": v }) }
fn stmt_if(t: Value, c: Vec<Value>, ei: Vec<Value>, a: Option<Vec<Value>>) -> Value {
    let mut o = json!({ "type": "Stmt.If", "test": t, "consequent": c, "elseIfs": ei, "trim": "none" });
    if let Some(a2) = a { o["alternate"] = json!(a2); }
    o
}
fn stmt_for(item: &str, idx: Option<&str>, iterable: Value, body: Vec<Value>) -> Value {
    let mut o = json!({ "type": "Stmt.For", "item": item, "iterable": iterable, "body": body, "trim": "none" });
    if let Some(i) = idx { o["index"] = json!(i); }
    o
}
fn stmt_block(n: &str, b: Vec<Value>) -> Value {
    json!({ "type": "Stmt.Block", "name": n, "body": b, "trim": "none" })
}
fn stmt_extends(p: Value) -> Value {
    json!({ "type": "Stmt.Extends", "parent": p, "trim": "none" })
}
fn stmt_include(p: Value) -> Value {
    json!({ "type": "Stmt.Include", "path": p, "trim": "none" })
}

// ---------------------------------------------------------------------------
// Node.js runtime stub — only for sanity-checking when node is present
// ---------------------------------------------------------------------------

const NODE_STUB: &str = r#"
const __rt = {
  htmlEscape: (s) => String(s).replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c])),
  valueToString: (v) => (v == null) ? '' : String(v),
  isTruthy: (v) => v !== null && v !== undefined && v !== '' && v !== 0 && v !== false,
  applyFilter: (name, v, args) => {
    switch (name) {
      case 'escape': case 'e': return String(v).replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));
      case 'raw': case 'safe': return v;
      default: return v;
    }
  },
  binary: (op, l, r) => {
    switch (op) { case '+': return l + r; default: return null; }
  },
  unary: (op, a) => { switch (op) { case '!': return !a; default: return null; } },
  exprMember: (o, p) => o == null ? null : o[p] ?? null,
  exprIndex: (a, i) => a == null ? null : a[i] ?? null,
};
const { htmlEscape, applyFilter, isTruthy, binary, unary, valueToString, exprMember, exprIndex } = __rt;
"#;

fn run_modules_in_node(modules: &[dejavu_codegen::GeneratedModule], ctx_js: &str) -> Option<String> {
    let mut stub = NODE_STUB.to_string();
    for m in modules {
        let base = m.path.rsplit('/').next().unwrap_or(&m.path).trim_end_matches(".js");
        let var = format!("__m_{base}");
        let fn_name = format!("render_{base}");
        let tmpl_name = format!("__tmpl_{base}"); // matches js_import_name("root:path") after sanitize
        let mut lines: Vec<&str> = m.code.lines().collect();
        lines.retain(|l| !l.trim().starts_with("import "));
        let body = lines.join("\n").replace("export function render", &format!("function {fn_name}"));
        stub.push_str(&format!(
            "// --- {} ---\n{body}\nconst {var} = {{ render: {fn_name} }};\nconst {tmpl_name} = {fn_name};\n\n",
            m.path
        ));
    }
    let entry = modules.first()?;
    let entry_base = entry.path.rsplit('/').next().unwrap_or(&entry.path).trim_end_matches(".js");
    let entry_var = format!("__m_{entry_base}");
    stub.push_str(&format!("console.log({entry_var}.render({ctx_js}, {{}}));" ));
    match std::process::Command::new("node").args(["-e", &stub]).output() {
        Ok(o) if o.status.success() => Some(String::from_utf8_lossy(&o.stdout).trim().to_string()),
        Ok(o) => { let stderr = String::from_utf8_lossy(&o.stderr); panic!("node fail: {stderr}"); }
        Err(_) => { eprintln!("SKIP: node absent"); None }
    }
}

// ---------------------------------------------------------------------------
// 1. include_single
// ---------------------------------------------------------------------------

#[test]
fn include_single_produces_two_modules_and_import() {
    let entry = doc(vec![text("hello "), stmt_include(lit(json!("footer")))]);
    let footer = doc(vec![text(" world!")]);
    let res = compile_catalog(
        &[("index.dejavu", entry), ("footer.dejavu", footer)],
        "index.dejavu",
    );
    assert_eq!(res.modules.len(), 2, "errors: {:?}", errors(&res));
    assert!(errors(&res).is_empty(), "errors: {:?}", errors(&res));

    let entry_m = &res.modules[0];
    assert_eq!(entry_m.path, "index.js");
    let internal = res.modules.iter().find(|m| m.path.starts_with("_internal/")).unwrap();
    assert!(internal.path.ends_with(".js"));

    let r = round(&entry_m.code);
    assert!(r.contains("import "));
    assert!(r.contains("_internal"));

    if let Some(out) = run_modules_in_node(&res.modules, "({})") {
        // render() joins strings — each push is a string.
        // Entry: "hello " then include footer which returns " world!"
        // __out.push("hello "); __out.push(footer.render(...)); → "hello  world!"
        assert_eq!(out, "hello  world!");
    }
}

// ---------------------------------------------------------------------------
// 2. extends_single_level
// ---------------------------------------------------------------------------

#[test]
fn extends_single_level_child_overrides_parent_block() {
    let parent = doc(vec![
        text("<header/>"),
        stmt_block("content", vec![text("default content")]),
        text("<footer/>"),
    ]);
    let child = doc(vec![
        stmt_extends(lit(json!("parent.dejavu"))),
        stmt_block("content", vec![text("overridden content")]),
    ]);
    let res = compile_catalog(
        &[("index.dejavu", child), ("parent.dejavu", parent)],
        "index.dejavu",
    );
    assert!(errors(&res).is_empty(), "errors: {:?}", errors(&res));
    let entry_m = &res.modules[0];
    let r = round(&entry_m.code);
    assert!(r.contains("__out.push(\"<header/>\");"));
    assert!(r.contains("__out.push(\"overridden content\");"));
    assert!(r.contains("__out.push(\"<footer/>\");"));
    assert!(!r.contains("default content"));

    if let Some(out) = run_modules_in_node(&res.modules, "({})") {
        assert_eq!(out, "<header/>overridden content<footer/>");
    }
}

// ---------------------------------------------------------------------------
// 3. extends_three_level
// ---------------------------------------------------------------------------

#[test]
fn extends_three_level_child_overrides_grandparent() {
    let gp = doc(vec![stmt_block("main", vec![text("gp main")])]);
    let parent = doc(vec![
        stmt_extends(lit(json!("grandparent.dejavu"))),
        stmt_block("main", vec![text("p main")]),
    ]);
    let child = doc(vec![
        stmt_extends(lit(json!("parent.dejavu"))),
        stmt_block("main", vec![text("c main")]),
    ]);
    let res = compile_catalog(
        &[("index.dejavu", child), ("parent.dejavu", parent), ("grandparent.dejavu", gp)],
        "index.dejavu",
    );
    assert!(errors(&res).is_empty(), "errors: {:?}", errors(&res));
    assert_eq!(res.modules.len(), 3);

    if let Some(out) = run_modules_in_node(&res.modules, "({})") {
        assert_eq!(out, "c main");
    }
}

// ---------------------------------------------------------------------------
// 4. block_without_override
// ---------------------------------------------------------------------------

#[test]
fn block_without_override_inherits_parent_default() {
    let parent = doc(vec![
        text("A-"),
        stmt_block("content", vec![text("parent default")]),
        text("-Z"),
    ]);
    let child = doc(vec![stmt_extends(lit(json!("parent.dejavu")))]);
    let res = compile_catalog(
        &[("index.dejavu", child), ("parent.dejavu", parent)],
        "index.dejavu",
    );
    let entry_m = &res.modules[0];
    let r = round(&entry_m.code);
    assert!(r.contains("parent default"));
    assert!(r.contains("A-"));
    assert!(r.contains("-Z"));

    if let Some(out) = run_modules_in_node(&res.modules, "({})") {
        assert_eq!(out, "A-parent default-Z");
    }
}

// ---------------------------------------------------------------------------
// 5. include_chain
// ---------------------------------------------------------------------------

#[test]
fn include_chain_a_include_b_include_c() {
    let c = doc(vec![text("C")]);
    let b = doc(vec![text("B("), stmt_include(lit(json!("c.dejavu"))), text(")")]);
    let a_body = doc(vec![text("A["), stmt_include(lit(json!("b.dejavu"))), text("]")]);
    let res = compile_catalog(
        &[("index.dejavu", a_body), ("b.dejavu", b), ("c.dejavu", c)],
        "index.dejavu",
    );
    assert!(errors(&res).is_empty(), "errors: {:?}", errors(&res));
    assert_eq!(res.modules.len(), 3);

    if let Some(out) = run_modules_in_node(&res.modules, "({})") {
        assert_eq!(out, "A[B(C)]");
    }
}

// ---------------------------------------------------------------------------
// 6. cycle_detected
// ---------------------------------------------------------------------------

#[test]
fn cycle_detected_include_ab_ba() {
    let a = doc(vec![text("A"), stmt_include(lit(json!("b.dejavu")))]);
    let b = doc(vec![text("B"), stmt_include(lit(json!("index.dejavu")))]);
    let res = compile_catalog(
        &[("index.dejavu", a), ("b.dejavu", b)],
        "index.dejavu",
    );
    assert!(res.modules.is_empty(), "cycle → no modules, got {:?}", res.modules.len());
    assert!(res.diagnostics.iter().any(|d| {
        d.code.contains("circular") && d.severity == dejavu_diagnostic::Severity::Error
    }));
}

// ---------------------------------------------------------------------------
// 7. mixed_if_for_include
// ---------------------------------------------------------------------------

#[test]
fn mixed_if_for_include_all_nodes_emit() {
    let include_tmpl = doc(vec![text("PART")]);
    let entry = doc(vec![
        text("pre "),
        stmt_if(
            ident("ok"),
            vec![text("YES "), interp(ident("name"), false)],
            vec![],
            Some(vec![text("NO ")]),
        ),
        text(" items=["),
        stmt_for("it", Some("i"), ident("items"), vec![interp(ident("it"), false), text(",")]),
        text("] "),
        stmt_include(lit(json!("include.dejavu"))),
    ]);
    let res = compile_catalog(
        &[("index.dejavu", entry), ("include.dejavu", include_tmpl)],
        "index.dejavu",
    );
    assert_eq!(res.modules.len(), 2);
    let entry_m = &res.modules[0];
    let r = round(&entry_m.code);
    assert!(r.contains("if (isTruthy(ctx.ok))"));
    assert!(r.contains("for (let __i = 0; __i < __it.length; __i++)"));
    assert!(r.contains("_internal"));

    if let Some(out) = run_modules_in_node(
        &res.modules,
        "({ok: true, name: 'Alice', items: ['x','y']})",
    ) {
        assert_eq!(out, "pre YES Alice items=[x,y,] PART");
    }
}
