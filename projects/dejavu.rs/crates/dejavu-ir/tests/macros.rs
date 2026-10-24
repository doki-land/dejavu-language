//! Macro declaration and invocation semantics in the shared renderer.

use std::collections::HashMap;

use dejavu_ir::{
    IrDocument, IrError, ResolvedTemplate, TemplateLoader, codes, default_language, diag_load,
    render_ir, render_ir_with_loader,
};
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

// ---------------------------------------------------------------------------
// `from ... import`
// ---------------------------------------------------------------------------

struct InMemoryLoader {
    docs: HashMap<String, IrDocument>,
}

impl InMemoryLoader {
    fn new() -> Self {
        Self { docs: HashMap::new() }
    }

    fn insert(mut self, path: &str, doc: IrDocument) -> Self {
        self.docs.insert(path.to_string(), doc);
        self
    }
}

impl TemplateLoader for InMemoryLoader {
    fn resolve(&mut self, ref_: &str, _from: Option<&str>) -> Result<ResolvedTemplate, IrError> {
        let doc = self
            .docs
            .get(ref_)
            .cloned()
            .ok_or_else(|| diag_load(codes::TEMPLATE_NOT_FOUND).arg("template", ref_))?;
        Ok(ResolvedTemplate { id: ref_.to_string(), document: doc })
    }
}

fn import_node(path: &str, names: Vec<(&str, Option<&str>)>) -> Value {
    let names: Vec<Value> = names
        .into_iter()
        .map(|(name, alias)| {
            let mut node = serde_json::json!({ "name": name });
            if let Some(alias) = alias {
                node["alias"] = serde_json::json!(alias);
            }
            node
        })
        .collect();
    serde_json::json!({
        "type": "Stmt.Import",
        "path": literal(serde_json::json!(path)),
        "names": names,
        "trim": "none",
    })
}

fn render_with_loader(entry: IrDocument, mut loader: InMemoryLoader) -> Result<String, IrError> {
    render_ir_with_loader(&entry, &serde_json::json!({}), &mut loader)
}

#[test]
fn imported_macro_binds_under_its_alias() {
    let lib = tpl(vec![macro_node(
        "greet",
        vec![param("name", None), param("mark", Some(literal(serde_json::json!("!"))))],
        vec![text("<em>"), interp(ident("name")), interp(ident("mark")), text("</em>")],
    )]);
    let main = tpl(vec![
        import_node("lib.dejavu", vec![("greet", Some("hi"))]),
        interp(call("hi", vec![literal(serde_json::json!("Ann"))])),
        text("|"),
        interp(call("hi", vec![literal(serde_json::json!("Bo")), literal(serde_json::json!("?"))])),
    ]);
    let out = render_with_loader(main, InMemoryLoader::new().insert("lib.dejavu", lib)).expect("render");
    assert_eq!(out, "<em>Ann!</em>|<em>Bo?</em>");
}

#[test]
fn imported_macro_body_resolves_against_its_own_module() {
    // The library macro calls a sibling macro that the importing template
    // never sees.
    let lib = tpl(vec![
        macro_node("inner", vec![param("v", None)], vec![text("["), interp(ident("v")), text("]")]),
        macro_node("outer", vec![param("v", None)], vec![interp(call("inner", vec![ident("v")]))]),
    ]);
    let main = tpl(vec![
        import_node("lib.dejavu", vec![("outer", None)]),
        interp(call("outer", vec![literal(serde_json::json!("x"))])),
    ]);
    let out = render_with_loader(main, InMemoryLoader::new().insert("lib.dejavu", lib)).expect("render");
    assert_eq!(out, "[x]");
}

#[test]
fn a_sibling_macro_of_an_imported_one_is_not_visible() {
    let lib = tpl(vec![
        macro_node("inner", vec![param("v", None)], vec![text("["), interp(ident("v")), text("]")]),
        macro_node("outer", vec![param("v", None)], vec![interp(call("inner", vec![ident("v")]))]),
    ]);
    let main = tpl(vec![
        import_node("lib.dejavu", vec![("outer", None)]),
        interp(call("inner", vec![literal(serde_json::json!("y"))])),
    ]);
    let err = render_with_loader(main, InMemoryLoader::new().insert("lib.dejavu", lib))
        .expect_err("sibling macro is not imported");
    assert_eq!(error_code(err), "dejavu.template.unknown-function");
}

#[test]
fn imported_macro_cannot_read_the_importing_context() {
    let lib = tpl(vec![macro_node("show", vec![], vec![interp(ident("outer"))])]);
    let main = tpl(vec![
        import_node("lib.dejavu", vec![("show", None)]),
        interp(call("show", vec![])),
    ]);
    let out = render_with_loader(main, InMemoryLoader::new().insert("lib.dejavu", lib)).expect("render");
    assert_eq!(out, "");
}

#[test]
fn importing_a_macro_the_target_does_not_declare_is_an_error() {
    let lib = tpl(vec![macro_node("greet", vec![], vec![text("x")])]);
    let main = tpl(vec![import_node("lib.dejavu", vec![("sign", None)])]);
    let err = render_with_loader(main, InMemoryLoader::new().insert("lib.dejavu", lib))
        .expect_err("unknown import");
    assert_eq!(error_code(err), "dejavu.template.unknown-import");
}

#[test]
fn importing_over_a_local_macro_name_is_an_error() {
    let lib = tpl(vec![macro_node("greet", vec![], vec![text("x")])]);
    let main = tpl(vec![
        macro_node("greet", vec![], vec![text("local")]),
        import_node("lib.dejavu", vec![("greet", None)]),
    ]);
    let err = render_with_loader(main, InMemoryLoader::new().insert("lib.dejavu", lib))
        .expect_err("conflict");
    assert_eq!(error_code(err), "dejavu.template.macro-conflict");
}
