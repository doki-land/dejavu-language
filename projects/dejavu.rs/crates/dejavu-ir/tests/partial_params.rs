//! Parameterized `partial` rendering: explicit binding, defaults, and errors.

use std::collections::HashMap;

use dejavu_ir::{
    IrDocument, IrError, ResolvedTemplate, TemplateLoader, codes, default_language, diag_load,
    render_ir_with_loader,
};
use serde_json::Value;

fn tpl(children: Vec<Value>) -> IrDocument {
    let doc_json = serde_json::json!({
        "irVersion": "v0",
        "language": default_language(),
        "body": { "type": "Template", "children": children }
    });
    serde_json::from_value(doc_json).expect("parse ir json")
}

fn text(value: &str) -> Value {
    serde_json::json!({ "type": "Text", "value": value })
}

fn interp(name: &str) -> Value {
    serde_json::json!({
        "type": "Interpolation",
        "expression": { "type": "Expr.Identifier", "name": name },
        "trim": "none",
        "raw": false,
    })
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

fn partial(params: Vec<Value>, body: Vec<Value>) -> Value {
    serde_json::json!({
        "type": "Stmt.Partial",
        "params": params,
        "body": body,
        "trim": "none",
    })
}

fn include(path: &str, with: Option<Value>) -> Value {
    let mut node = serde_json::json!({
        "type": "Stmt.Include",
        "path": { "type": "Expr.Literal", "value": path },
        "trim": "none",
    });
    if let Some(w) = with {
        node["with"] = w;
    }
    node
}

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

fn render(entry: &IrDocument, ctx: Value, loader: &mut InMemoryLoader) -> Result<String, IrError> {
    render_ir_with_loader(entry, &ctx, loader)
}

fn code(err: IrError) -> String {
    match err {
        IrError::Diagnostic(d) => d.code,
        other => panic!("expected diagnostic, got {other:?}"),
    }
}

#[test]
fn partial_uses_declared_default_when_argument_is_absent() {
    let entry = tpl(vec![include("card", Some(literal(serde_json::json!({ "title": "Hi" }))))]);
    let card = tpl(vec![partial(
        vec![param("title", None), param("suffix", Some(literal(serde_json::json!("!"))))],
        vec![text("["), interp("title"), interp("suffix"), text("]")],
    )]);
    let mut loader = InMemoryLoader::new().insert("card", card);

    let out = render(&entry, serde_json::json!({}), &mut loader).expect("render");
    assert_eq!(out, "[Hi!]");
}

#[test]
fn partial_uses_explicit_argument_over_default() {
    let entry = tpl(vec![include("card", Some(literal(serde_json::json!({ "suffix": "?" }))))]);
    let card = tpl(vec![partial(
        vec![param("suffix", Some(literal(serde_json::json!("!"))))],
        vec![interp("suffix")],
    )]);
    let mut loader = InMemoryLoader::new().insert("card", card);

    let out = render(&entry, serde_json::json!({}), &mut loader).expect("render");
    assert_eq!(out, "?");
}

#[test]
fn partial_rejects_missing_required_parameter() {
    let entry = tpl(vec![include("card", None)]);
    let card = tpl(vec![partial(vec![param("title", None)], vec![interp("title")])]);
    let mut loader = InMemoryLoader::new().insert("card", card);

    let err = render(&entry, serde_json::json!({ "title": "caller" }), &mut loader)
        .expect_err("missing parameter must fail");
    assert_eq!(code(err), codes::MISSING_PARAMETER);
}

#[test]
fn partial_rejects_undeclared_parameter() {
    let entry = tpl(vec![include("card", Some(literal(serde_json::json!({ "extra": 1 }))))]);
    let card = tpl(vec![partial(
        vec![param("title", Some(literal(serde_json::json!("T"))))],
        vec![interp("title")],
    )]);
    let mut loader = InMemoryLoader::new().insert("card", card);

    let err = render(&entry, serde_json::json!({}), &mut loader)
        .expect_err("unknown parameter must fail");
    assert_eq!(code(err), codes::UNKNOWN_PARAMETER);
}

#[test]
fn partial_does_not_inherit_caller_context() {
    let entry = tpl(vec![include(
        "card",
        Some(literal(serde_json::json!({ "title": "T" }))),
    )]);
    let card = tpl(vec![partial(
        vec![param("title", None)],
        vec![interp("title"), text("/"), interp("name")],
    )]);
    let mut loader = InMemoryLoader::new().insert("card", card);

    let out = render(&entry, serde_json::json!({ "name": "caller" }), &mut loader).expect("render");
    assert_eq!(out, "T/");
}
