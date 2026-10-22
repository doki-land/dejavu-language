//! Inheritance conformance tests — extends / include / block / super.
//!
//! 直接用 IR JSON 构造模板文档，绕过 parser 尚未实现的
//! extends/include/block/super 语句，专门验证 render.rs 的 TemplateLoader
//! + 继承链 + block override + super 逻辑。

use std::collections::HashMap;

use dejavu_ir::{IrDocument, IrError, TemplateLoader, default_language, render_ir_with_loader};
use serde_json::Value;

fn tpl_ir(children: Vec<serde_json::Value>) -> IrDocument {
    let doc_json = serde_json::json!({
        "irVersion": "1.0",
        "language": default_language(),
        "body": {
            "type": "Template",
            "children": children,
        }
    });
    serde_json::from_value(doc_json).expect("parse ir json")
}

fn text_node(value: &str) -> serde_json::Value {
    serde_json::json!({ "type": "Text", "value": value })
}

fn interpolation(expr: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "type": "Interpolation",
        "expression": expr,
        "trim": "none",
        "raw": false,
    })
}

fn literal_str(s: &str) -> serde_json::Value {
    serde_json::json!({ "type": "Expr.Literal", "value": s })
}

fn block_node(name: &str, body: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "type": "Stmt.Block",
        "name": name,
        "body": body,
        "trim": "none",
    })
}

fn extends_node(path: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "Stmt.Extends",
        "parent": literal_str(path),
        "trim": "none",
    })
}

fn include_node(path: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "Stmt.Include",
        "path": literal_str(path),
        "trim": "none",
    })
}

fn super_node() -> serde_json::Value {
    serde_json::json!({ "type": "Stmt.Super", "trim": "none" })
}

// ---------------------------------------------------------------------------
// FileLoader 直接从 IR 文档表查（绕过 parser）
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
    fn load(&mut self, path: &str) -> Result<IrDocument, IrError> {
        self.docs
            .get(path)
            .cloned()
            .ok_or_else(|| IrError::Render(format!("template not found: {path}")))
    }
}

// ===========================================================================
// 测试 1：extends + block override
// ===========================================================================

#[test]
fn extends_block_override() {
    // base.dejavu: "<html><body><% block content %>default<% end %></body></html>"
    let base = tpl_ir(vec![
        text_node("<html><body>"),
        block_node("content", vec![text_node("default")]),
        text_node("</body></html>"),
    ]);

    // child.dejavu: "<% extends 'base.dejavu' %><% block content %>hello<% end %>"
    let child = tpl_ir(vec![
        extends_node("base.dejavu"),
        block_node("content", vec![text_node("hello")]),
    ]);

    let loader = InMemoryLoader::new()
        .insert("base.dejavu", base)
        .insert("child.dejavu", child);

    // 需要 &mut 但 InMemoryLoader 不是 Copy/Clone —— 重构
    let mut loader = loader;
    let entry = loader.load("child.dejavu").unwrap();
    let output = render_ir_with_loader(
        &entry,
        &Value::Object(serde_json::Map::new()),
        &mut loader,
    ).unwrap();

    assert_eq!(output, "<html><body>hello</body></html>");
}

// ===========================================================================
// 测试 2：include
// ===========================================================================

#[test]
fn include_partial() {
    // partial.dejavu: "<!-- nav -->"
    let partial = tpl_ir(vec![text_node("<!-- nav -->")]);

    // main.dejavu: "<html><% include 'partial.dejavu' %></html>"
    let main = tpl_ir(vec![
        text_node("<html>"),
        include_node("partial.dejavu"),
        text_node("</html>"),
    ]);

    let mut loader = InMemoryLoader::new()
        .insert("partial.dejavu", partial)
        .insert("main.dejavu", main);

    let entry = loader.load("main.dejavu").unwrap();
    let output = render_ir_with_loader(
        &entry,
        &Value::Object(serde_json::Map::new()),
        &mut loader,
    ).unwrap();

    assert_eq!(output, "<html><!-- nav --></html>");
}

// ===========================================================================
// 测试 3：super 在 child block 中引用 parent 原始内容
// ===========================================================================

#[test]
fn super_renders_parent_block_body() {
    // base.dejavu: "A<% block body %>original<% end %>B"
    let base = tpl_ir(vec![
        text_node("A"),
        block_node("body", vec![text_node("original")]),
        text_node("B"),
    ]);

    // child.dejavu: "<% extends 'base.dejavu' %><% block body %>before <% super %> after<% end %>"
    let child = tpl_ir(vec![
        extends_node("base.dejavu"),
        block_node("body", vec![
            text_node("before "),
            super_node(),
            text_node(" after"),
        ]),
    ]);

    let mut loader = InMemoryLoader::new()
        .insert("base.dejavu", base)
        .insert("child.dejavu", child);

    let entry = loader.load("child.dejavu").unwrap();
    let output = render_ir_with_loader(
        &entry,
        &Value::Object(serde_json::Map::new()),
        &mut loader,
    ).unwrap();

    assert_eq!(output, "Abefore original afterB");
}

// ===========================================================================
// 测试 4：block override 中无 super → 完全覆盖
// ===========================================================================

#[test]
fn block_override_without_super_replaces() {
    // base.dejavu: "<% block title %>Base<% end %>"
    let base = tpl_ir(vec![
        block_node("title", vec![text_node("Base")]),
    ]);

    // child.dejavu: "<% extends 'base.dejavu' %><% block title %>Override<% end %>"
    let child = tpl_ir(vec![
        extends_node("base.dejavu"),
        block_node("title", vec![text_node("Override")]),
    ]);

    let mut loader = InMemoryLoader::new()
        .insert("base.dejavu", base)
        .insert("child.dejavu", child);

    let entry = loader.load("child.dejavu").unwrap();
    let output = render_ir_with_loader(
        &entry,
        &Value::Object(serde_json::Map::new()),
        &mut loader,
    ).unwrap();

    assert_eq!(output, "Override");
}

// ===========================================================================
// 测试 5：child 没有定义某个 block → 使用 parent 原始内容
// ===========================================================================

#[test]
fn missing_block_in_child_falls_back_to_parent() {
    // base.dejavu: "<% block a %>AAA<% end %><% block b %>BBB<% end %>"
    let base = tpl_ir(vec![
        block_node("a", vec![text_node("AAA")]),
        block_node("b", vec![text_node("BBB")]),
    ]);

    // child.dejavu: "<% extends 'base.dejavu' %><% block a %>overridden<% end %>"
    let child = tpl_ir(vec![
        extends_node("base.dejavu"),
        block_node("a", vec![text_node("overridden")]),
    ]);

    let mut loader = InMemoryLoader::new()
        .insert("base.dejavu", base)
        .insert("child.dejavu", child);

    let entry = loader.load("child.dejavu").unwrap();
    let output = render_ir_with_loader(
        &entry,
        &Value::Object(serde_json::Map::new()),
        &mut loader,
    ).unwrap();

    assert_eq!(output, "overriddenBBB");
}

// ===========================================================================
// 测试 6：循环 extends 检测
// ===========================================================================

#[test]
fn circular_extends_is_detected() {
    let a = tpl_ir(vec![extends_node("b.dejavu")]);
    let b = tpl_ir(vec![extends_node("a.dejavu")]);

    let mut loader = InMemoryLoader::new()
        .insert("a.dejavu", a)
        .insert("b.dejavu", b);

    let entry = loader.load("a.dejavu").unwrap();
    let result = render_ir_with_loader(
        &entry,
        &Value::Object(serde_json::Map::new()),
        &mut loader,
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("circular"));
}

// ===========================================================================
// 测试 7：render_ir（无 loader）遇到 include/extends 必须报错
// ===========================================================================

#[test]
fn render_ir_without_loader_errors_on_include() {
    let main = tpl_ir(vec![include_node("partial.dejavu")]);
    let result = dejavu_ir::render_ir(
        &main,
        &Value::Object(serde_json::Map::new()),
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("requires a template loader"));
}
