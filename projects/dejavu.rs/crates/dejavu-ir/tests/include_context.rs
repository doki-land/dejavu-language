//! `include` context semantics: inherit, `with`, and `only` / `without context`.

use std::collections::HashMap;

use dejavu_ir::{
    IrDocument, IrError, ResolvedTemplate, Stage, TemplateLoader, codes, default_language,
    diag_load, render_ir_with_loader,
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

fn interp_expr(expression: Value) -> Value {
    serde_json::json!({
        "type": "Interpolation",
        "expression": expression,
        "trim": "none",
        "raw": false,
    })
}

fn interp(name: &str) -> Value {
    interp_expr(identifier(name))
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

fn member(object: Value, property: &str) -> Value {
    serde_json::json!({ "type": "Expr.Member", "object": object, "property": property })
}

fn object(entries: &[(&str, Value)]) -> Value {
    let entries: Vec<Value> = entries
        .iter()
        .map(|(key, value)| serde_json::json!({ "key": key, "value": value }))
        .collect();
    serde_json::json!({ "type": "Expr.Object", "entries": entries })
}

fn stmt_if(test: Value, consequent: Vec<Value>) -> Value {
    serde_json::json!({
        "type": "Stmt.If",
        "test": test,
        "consequent": consequent,
        "elseIfs": [],
        "trim": "none",
    })
}

fn stmt_partial(params: &[(&str, Option<Value>)], body: Vec<Value>) -> Value {
    let params: Vec<Value> = params
        .iter()
        .map(|(name, default)| match default {
            Some(default) => serde_json::json!({ "name": name, "default": default }),
            None => serde_json::json!({ "name": name }),
        })
        .collect();
    serde_json::json!({
        "type": "Stmt.Partial",
        "params": params,
        "body": body,
        "trim": "none",
    })
}

/// A `partial node` that renders `node.name` and recurses into `node.child`.
fn recursive_partial() -> IrDocument {
    tpl(vec![stmt_partial(
        &[("node", None)],
        vec![
            interp_expr(member(identifier("node"), "name")),
            stmt_if(
                member(identifier("node"), "child"),
                vec![
                    text("("),
                    include(
                        "tree",
                        Some(object(&[("node", member(identifier("node"), "child"))])),
                        false,
                    ),
                    text(")"),
                ],
            ),
        ],
    )])
}

struct InMemoryLoader {
    docs: HashMap<String, IrDocument>,
    resolves: usize,
}

impl InMemoryLoader {
    fn new() -> Self {
        Self { docs: HashMap::new(), resolves: 0 }
    }

    fn insert(mut self, path: &str, doc: IrDocument) -> Self {
        self.docs.insert(path.to_string(), doc);
        self
    }
}

impl TemplateLoader for InMemoryLoader {
    fn resolve(&mut self, ref_: &str, _from: Option<&str>) -> Result<ResolvedTemplate, IrError> {
        self.resolves += 1;
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

#[test]
fn partial_reenters_itself_to_render_nested_data() {
    let entry = tpl(vec![include(
        "tree",
        Some(object(&[("node", identifier("node"))])),
        false,
    )]);
    let mut loader = InMemoryLoader::new().insert("tree", recursive_partial());

    let ctx = serde_json::json!({
        "node": {
            "name": "root",
            "child": { "name": "a", "child": { "name": "a1", "child": { "name": "a1x" } } }
        }
    });
    let out = render(&entry, ctx, &mut loader).expect("render");
    assert_eq!(out, "root(a(a1(a1x)))");
}

#[test]
fn partial_recursion_limit_reports_render_diagnostic() {
    // `partial node` that includes itself without any base case.
    let looping = tpl(vec![stmt_partial(
        &[("node", None)],
        vec![include(
            "loop",
            Some(object(&[("node", identifier("node"))])),
            false,
        )],
    )]);
    let entry = tpl(vec![include(
        "loop",
        Some(object(&[("node", literal(serde_json::json!(1)))])),
        false,
    )]);
    let mut loader = InMemoryLoader::new().insert("loop", looping);

    let err = render(&entry, serde_json::json!({}), &mut loader).expect_err("unbounded recursion must fail");
    match err {
        IrError::Diagnostic(d) => {
            assert_eq!(d.code, codes::PARTIAL_RECURSION);
            assert_eq!(d.stage, Stage::Render);
            assert_eq!(d.args.get("limit").cloned(), Some(32.0.into()));
        }
        other => panic!("expected diagnostic, got {other:?}"),
    }
}

#[test]
fn repeated_include_resolves_once_per_render() {
    let entry = tpl(vec![
        include("partial", None, false),
        include("partial", None, false),
        include("partial", None, false),
    ]);
    let partial = tpl(vec![text("x")]);
    let mut loader = InMemoryLoader::new().insert("partial", partial);

    let out = render(&entry, serde_json::json!({}), &mut loader).expect("render");
    assert_eq!(out, "xxx");
    assert_eq!(loader.resolves, 1, "same (ref, from) must resolve once");
}
