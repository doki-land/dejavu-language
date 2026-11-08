//! 共享渲染预算的越限用例：源 / 输出 / include 深度 / 步数。

use std::collections::HashMap;

use dejavu_ir::{
    IrDocument, IrError, RenderLimits, RenderOptions, ResolvedTemplate, TemplateLoader, check_source_budget, codes, default_language,
    diag_load, render_ir_opts,
};

fn tpl_ir(children: Vec<serde_json::Value>) -> IrDocument {
    let doc_json = serde_json::json!({
        "irVersion": "v0",
        "language": default_language(),
        "body": { "type": "Template", "children": children }
    });
    serde_json::from_value(doc_json).expect("parse ir json")
}

fn text_node(value: &str) -> serde_json::Value {
    serde_json::json!({ "type": "Text", "value": value })
}

fn include_node(path: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "Stmt.Include",
        "path": { "type": "Expr.Literal", "value": path },
        "trim": "none",
    })
}

struct InMemoryLoader {
    docs: HashMap<String, IrDocument>,
}

impl TemplateLoader for InMemoryLoader {
    fn resolve(&mut self, ref_: &str, _from: Option<&str>) -> Result<ResolvedTemplate, IrError> {
        let doc = self.docs.get(ref_).cloned().ok_or_else(|| diag_load(codes::TEMPLATE_NOT_FOUND).arg("template", ref_))?;
        Ok(ResolvedTemplate { id: ref_.to_string(), document: doc })
    }
}

fn render(doc: &IrDocument, loader: &mut InMemoryLoader, limits: RenderLimits) -> Result<String, IrError> {
    let ctx = serde_json::json!({});
    render_ir_opts(doc, &ctx, RenderOptions { limits, ..Default::default() }, Some(loader))
}

fn code_of(error: IrError) -> String {
    match error {
        IrError::Diagnostic(diagnostic) => diagnostic.code,
        IrError::Json(_) => panic!("expected a diagnostic"),
    }
}

#[test]
fn source_budget_rejects_oversized_source() {
    let limits = RenderLimits { max_source_bytes: 4, ..RenderLimits::default() };
    let diagnostic = check_source_budget("hello", &limits).unwrap_err();
    assert_eq!(diagnostic.code, codes::SOURCE_BUDGET);
    assert!(check_source_budget("abc", &limits).is_ok());
}

#[test]
fn step_budget_rejects_before_the_next_node() {
    let doc = tpl_ir(vec![text_node("a"), text_node("b")]);
    let mut loader = InMemoryLoader { docs: HashMap::new() };
    let limits = RenderLimits { max_steps: 1, ..RenderLimits::default() };
    assert_eq!(code_of(render(&doc, &mut loader, limits).unwrap_err()), codes::STEP_BUDGET);
}

#[test]
fn output_budget_rejects_oversized_output() {
    let doc = tpl_ir(vec![text_node("hello")]);
    let mut loader = InMemoryLoader { docs: HashMap::new() };
    let limits = RenderLimits { max_output_bytes: 3, ..RenderLimits::default() };
    assert_eq!(code_of(render(&doc, &mut loader, limits).unwrap_err()), codes::OUTPUT_BUDGET);
}

#[test]
fn include_depth_budget_rejects_deep_chains() {
    let mut docs = HashMap::new();
    docs.insert("main".to_string(), tpl_ir(vec![include_node("a")]));
    docs.insert("a".to_string(), tpl_ir(vec![include_node("b")]));
    docs.insert("b".to_string(), tpl_ir(vec![text_node("b")]));
    let mut loader = InMemoryLoader { docs };
    let limits = RenderLimits { max_include_depth: 1, ..RenderLimits::default() };
    assert_eq!(code_of(render(&docs_main(), &mut loader, limits).unwrap_err()), codes::INCLUDE_DEPTH);
}

fn docs_main() -> IrDocument {
    tpl_ir(vec![include_node("a")])
}
