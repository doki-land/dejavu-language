//! `include` context semantics: inherit, `with`, and `only` / `without context`.

use std::collections::HashMap;

use dejavu_ir::{
    IrDocument, IrError, ResolvedTemplate, TemplateLoader, codes, default_language, diag_load,
    render_ir_with_loader,
};
use serde_json::Value;

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

fn interp(name: &str) -> Value {
    serde_json::json!({
        "type": "Interpolation",
        "expression": { "type": "Expr.Identifier", "name": name },
        "trim": "none",
        "raw": false,
    })
}

fn include(path: &str, with: Option<Value>, only: bool) -> Value {
    let mut node = serde_json::json!({
        "type": "Stmt.Include",
        "path": { "type": "Expr.Literal", "value": path },
        "trim": "none",
        "only": only,
    });
    if let Some(w) = with {
        node["with"] = w;
    }
    node
}

fn literal(value: Value) -> Value {
    serde_json::json!({ "type": "Expr.Literal", "value": value })
}

fn identifier(name: &str) -> Value {
    serde_json::json!({ "type": "Expr.Identifier", "name": name })
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

#[test]
fn include_inherits_caller_context_by_default() {
    let entry = tpl(vec![text("A-"), include("partial", None, false), text("-Z")]);
    let partial = tpl(vec![text("partial:"), interp("name")]);
    let mut loader = InMemoryLoader::new().insert("partial", partial);

    let out = render(&entry, serde_json::json!({ "name": "X" }), &mut loader).expect("render");
    assert_eq!(out, "A-partial:X-Z");
}

#[test]
fn include_with_merges_explicit_context_over_caller() {
    let entry = tpl(vec![
        text("A-"),
        include("partial", Some(identifier("card")), false),
        text("-Z"),
    ]);
    let partial = tpl(vec![interp("name"), text(":"), interp("title")]);
    let mut loader = InMemoryLoader::new().insert("partial", partial);

    let ctx = serde_json::json!({ "name": "X", "card": { "title": "T" } });
    let out = render(&entry, ctx, &mut loader).expect("render");
    assert_eq!(out, "A-X:T-Z");
}

#[test]
fn include_with_overrides_inherited_entries() {
    let entry = tpl(vec![include("partial", Some(identifier("card")), false)]);
    let partial = tpl(vec![interp("name")]);
    let mut loader = InMemoryLoader::new().insert("partial", partial);

    let ctx = serde_json::json!({ "name": "X", "card": { "name": "Y" } });
    let out = render(&entry, ctx, &mut loader).expect("render");
    assert_eq!(out, "Y");
}

#[test]
fn include_only_isolates_the_caller_context() {
    let entry = tpl(vec![
        text("A-"),
        include("partial", None, true),
        text("-"),
        interp("name"),
        text("-Z"),
    ]);
    let partial = tpl(vec![text("partial:"), interp("name")]);
    let mut loader = InMemoryLoader::new().insert("partial", partial);

    let out = render(&entry, serde_json::json!({ "name": "X" }), &mut loader).expect("render");
    assert_eq!(out, "A-partial:-X-Z");
}

#[test]
fn include_with_only_exposes_only_the_explicit_context() {
    let entry = tpl(vec![include("partial", Some(identifier("card")), true)]);
    let partial = tpl(vec![interp("name"), text(":"), interp("title")]);
    let mut loader = InMemoryLoader::new().insert("partial", partial);

    let ctx = serde_json::json!({ "name": "X", "card": { "title": "T" } });
    let out = render(&entry, ctx, &mut loader).expect("render");
    assert_eq!(out, ":T");
}

#[test]
fn include_with_literal_object_context() {
    let entry = tpl(vec![include("partial", Some(literal(serde_json::json!({ "name": "L" }))), true)]);
    let partial = tpl(vec![interp("name")]);
    let mut loader = InMemoryLoader::new().insert("partial", partial);

    let out = render(&entry, serde_json::json!({ "name": "X" }), &mut loader).expect("render");
    assert_eq!(out, "L");
}

#[test]
fn include_with_non_object_context_is_rejected() {
    let entry = tpl(vec![include("partial", Some(literal(serde_json::json!("nope"))), false)]);
    let partial = tpl(vec![text("unused")]);
    let mut loader = InMemoryLoader::new().insert("partial", partial);

    let err = render(&entry, serde_json::json!({}), &mut loader).expect_err("string context must fail");
    match err {
        IrError::Diagnostic(d) => assert_eq!(d.code, codes::CONTEXT_NOT_OBJECT),
        other => panic!("expected diagnostic, got {other:?}"),
    }
}

#[test]
fn include_with_null_context_behaves_like_no_entries() {
    let entry = tpl(vec![include("partial", Some(literal(Value::Null)), false)]);
    let partial = tpl(vec![interp("name")]);
    let mut loader = InMemoryLoader::new().insert("partial", partial);

    let out = render(&entry, serde_json::json!({ "name": "X" }), &mut loader).expect("render");
    assert_eq!(out, "X");
}
