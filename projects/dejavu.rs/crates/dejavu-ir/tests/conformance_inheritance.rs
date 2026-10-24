//! Inheritance conformance tests — extends / include / block / super.
//!
//! 直接用 IR JSON 构造模板文档，绕过 parser 尚未实现的
//! extends/include/block/super 语句，专门验证 render.rs 的 TemplateLoader
//! + 继承链 + block override + super 逻辑。

use std::collections::HashMap;

use dejavu_ir::{
    IrDocument, IrError, RenderOptions, ResolvedTemplate, TemplateLoader, codes, default_language, diag_load,
    render_ir_opts, render_ir_with_loader,
};
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

fn identifier_node(name: &str) -> serde_json::Value {
    serde_json::json!({ "type": "Expr.Identifier", "name": name })
}

fn pipe_node(expression: serde_json::Value, filter: &str, arguments: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "type": "Expr.Pipe",
        "expression": expression,
        "filter": filter,
        "arguments": arguments,
    })
}

fn interpolation_identifier(name: &str) -> serde_json::Value {
    interpolation(identifier_node(name))
}

fn interpolation_pipe(identifier: &str, filter: &str, args: Vec<serde_json::Value>) -> serde_json::Value {
    interpolation(pipe_node(identifier_node(identifier), filter, args))
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
    fn resolve(&mut self, ref_: &str, _from: Option<&str>) -> Result<ResolvedTemplate, IrError> {
        let doc = self
            .docs
            .get(ref_)
            .cloned()
            .ok_or_else(|| diag_load(codes::TEMPLATE_NOT_FOUND).arg("template", ref_))?;
        Ok(ResolvedTemplate {
            id: ref_.to_string(),
            document: doc,
        })
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
    let entry = (loader.resolve("child.dejavu", None).unwrap().document);
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

    let entry = (loader.resolve("main.dejavu", None).unwrap().document);
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

    let entry = (loader.resolve("child.dejavu", None).unwrap().document);
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

    let entry = (loader.resolve("child.dejavu", None).unwrap().document);
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

    let entry = (loader.resolve("child.dejavu", None).unwrap().document);
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

    let entry = (loader.resolve("a.dejavu", None).unwrap().document);
    let result = render_ir_with_loader(
        &entry,
        &Value::Object(serde_json::Map::new()),
        &mut loader,
    );

    assert!(result.is_err());
    match result.unwrap_err() {
        IrError::Diagnostic(d) => assert_eq!(d.code, codes::CIRCULAR_EXTENDS),
        other => panic!("expected Diagnostic, got {other:?}"),
    }
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
    match result.unwrap_err() {
        IrError::Diagnostic(d) => {
            assert_eq!(d.code, codes::MISSING_LOADER);
            assert_eq!(
                d.args.get("feature"),
                Some(&dejavu_diagnostic::ArgValue::String("include".into()))
            );
        }
        other => panic!("expected Diagnostic, got {other:?}"),
    }
}

// ===========================================================================
// strictUndefined：缺失变量在 strict 模式下报错
// ===========================================================================

#[test]
fn strict_undefined_throws_on_missing_var() {
    let doc = tpl_ir(vec![interpolation_identifier("missing")]);
    let ctx = Value::Object(serde_json::Map::new());
    let opts = RenderOptions {
        strict_undefined: true,
        name: Some("t".into()),
        ..Default::default()
    };
    let result = render_ir_opts(&doc, &ctx, opts, None);
    assert!(result.is_err());
    match result.unwrap_err() {
        IrError::Diagnostic(d) => {
            assert_eq!(d.code, codes::UNDEFINED_VARIABLE);
            assert_eq!(
                d.args.get("name"),
                Some(&dejavu_diagnostic::ArgValue::String("missing".into()))
            );
        }
        other => panic!("expected Diagnostic, got {other:?}"),
    }
}

#[test]
fn strict_undefined_disabled_returns_null_as_empty() {
    let doc = tpl_ir(vec![interpolation_identifier("missing")]);
    let ctx = Value::Object(serde_json::Map::new());
    let opts = RenderOptions {
        strict_undefined: false,
        name: Some("t".into()),
        ..Default::default()
    };
    let out = render_ir_opts(&doc, &ctx, opts, None).unwrap();
    assert_eq!(out, "");
}

// ===========================================================================
// escape/e filter：内部 html_escape + is_raw_filter 跳过二次 escape
// ===========================================================================

#[test]
fn escape_filter_does_not_double_escape() {
    let doc = tpl_ir(vec![interpolation_pipe("val", "escape", vec![])]);
    let mut ctx = serde_json::Map::new();
    ctx.insert("val".into(), Value::String("<b>hi</b>".into()));
    let opts = RenderOptions::default();
    let out = render_ir_opts(&doc, &Value::Object(ctx), opts, None).unwrap();
    assert_eq!(out, "&lt;b&gt;hi&lt;/b&gt;");
}

#[test]
fn e_alias_filter_works_like_escape() {
    let doc = tpl_ir(vec![interpolation_pipe("val", "e", vec![])]);
    let mut ctx = serde_json::Map::new();
    ctx.insert("val".into(), Value::String("a & b".into()));
    let opts = RenderOptions::default();
    let out = render_ir_opts(&doc, &Value::Object(ctx), opts, None).unwrap();
    assert_eq!(out, "a &amp; b");
}

#[test]
fn default_interpolation_does_html_escape() {
    let doc = tpl_ir(vec![interpolation_identifier("val")]);
    let mut ctx = serde_json::Map::new();
    ctx.insert("val".into(), Value::String("<x>".into()));
    let opts = RenderOptions::default();
    let out = render_ir_opts(&doc, &Value::Object(ctx), opts, None).unwrap();
    assert_eq!(out, "&lt;x&gt;");
}

// ===========================================================================
// date filter：ISO / epoch ms
// ===========================================================================

#[test]
fn date_filter_iso_string() {
    let doc = tpl_ir(vec![interpolation_pipe("d", "date", vec![])]);
    let mut ctx = serde_json::Map::new();
    ctx.insert("d".into(), Value::String("2024-03-15".into()));
    let opts = RenderOptions::default();
    let out = render_ir_opts(&doc, &Value::Object(ctx), opts, None).unwrap();
    assert_eq!(out, "2024-03-15");
}

#[test]
fn date_filter_epoch_ms() {
    // 1705276800000 ms → 2024-01-15 00:00:00 UTC
    let doc = tpl_ir(vec![interpolation_pipe("d", "date", vec![])]);
    let mut ctx = serde_json::Map::new();
    ctx.insert("d".into(), Value::String("1705276800000".into()));
    let opts = RenderOptions::default();
    let out = render_ir_opts(&doc, &Value::Object(ctx), opts, None).unwrap();
    assert_eq!(out, "2024-01-15");
}

// ===========================================================================
// slug filter：Unicode 重音折叠 + 小写 + hyphen 压缩
// ===========================================================================

#[test]
fn slug_filter_folds_accents_and_compacts() {
    let doc = tpl_ir(vec![interpolation_pipe("s", "slug", vec![])]);
    let mut ctx = serde_json::Map::new();
    ctx.insert("s".into(), Value::String("Café Résumé".into()));
    let opts = RenderOptions::default();
    let out = render_ir_opts(&doc, &Value::Object(ctx), opts, None).unwrap();
    assert_eq!(out, "cafe-resume");
}

#[test]
fn slug_filter_trims_edge_hyphens() {
    let doc = tpl_ir(vec![interpolation_pipe("s", "slug", vec![])]);
    let mut ctx = serde_json::Map::new();
    ctx.insert("s".into(), Value::String("!!! Hello World ???".into()));
    let opts = RenderOptions::default();
    let out = render_ir_opts(&doc, &Value::Object(ctx), opts, None).unwrap();
    assert_eq!(out, "hello-world");
}

// ===========================================================================
// include 循环检测：双向 A include B include A
// ===========================================================================

#[test]
fn circular_include_abba_detected() {
    let a = tpl_ir(vec![text_node("A("), include_node("b"), text_node(")")]);
    let b = tpl_ir(vec![text_node("B("), include_node("a"), text_node(")")]);

    let mut loader = InMemoryLoader::new()
        .insert("a", a)
        .insert("b", b);

    let entry = loader.resolve("a", None).unwrap().document;
    let result = render_ir_with_loader(
        &entry,
        &Value::Object(serde_json::Map::new()),
        &mut loader,
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        IrError::Diagnostic(d) => assert_eq!(d.code, codes::CIRCULAR_INCLUDE),
        other => panic!("expected Diagnostic, got {other:?}"),
    }
}

// ===========================================================================
// extends 循环 canonical id：同模板不同路径写法仍被识别
// ===========================================================================

/// 测试专用 loader：把 "<canonical_id>:<any_path>" 这种 ref 解到同一个 id。
struct AliasLoader {
    docs: HashMap<String, IrDocument>,
}

impl AliasLoader {
    fn new() -> Self {
        Self { docs: HashMap::new() }
    }
    fn insert(mut self, id: &str, doc: IrDocument) -> Self {
        self.docs.insert(id.to_string(), doc);
        self
    }
}

impl TemplateLoader for AliasLoader {
    fn resolve(&mut self, ref_: &str, _from: Option<&str>) -> Result<ResolvedTemplate, IrError> {
        let canonical_id = ref_
            .split_once(':')
            .map(|(id, _)| id.to_string())
            .unwrap_or_else(|| ref_.to_string());
        let doc = self
            .docs
            .get(&canonical_id)
            .cloned()
            .ok_or_else(|| diag_load(codes::TEMPLATE_NOT_FOUND).arg("template", ref_))?;
        Ok(ResolvedTemplate {
            id: canonical_id,
            document: doc,
        })
    }
}

#[test]
fn extends_cycle_via_canonical_id_alias() {
    // parent extends "parent:parent.alias" → canonical id "parent"
    // child extends "parent:real/parent.dejavu" → 同一个 canonical id "parent"
    // extends 循环必须在 canonical id 层面识别，不能靠路径字符串
    let parent = tpl_ir(vec![extends_node("parent:parent.alias"), text_node("parent-body")]);
    let child = tpl_ir(vec![extends_node("parent:real/parent.dejavu"), text_node("child-body")]);

    let mut loader = AliasLoader::new()
        .insert("parent", parent)
        .insert("child", child);

    let entry = loader.resolve("child", None).unwrap().document;
    let result = render_ir_with_loader(
        &entry,
        &Value::Object(serde_json::Map::new()),
        &mut loader,
    );
    assert!(result.is_err(), "should detect canonical-id-level extends cycle");
    match result.unwrap_err() {
        IrError::Diagnostic(d) => assert_eq!(d.code, codes::CIRCULAR_EXTENDS),
        other => panic!("expected Diagnostic, got {other:?}"),
    }
}

// ===========================================================================
// 以下 5 个测试是 2026-10-23 修复的 Rust renderer correctness 回归测试
// ===========================================================================

/// 三层 extends（child → middle → base）：最底层 child block 最终覆盖全部 parent。
/// 修复前的 bug：递归时 self.blocks 被上层 child_blocks 覆盖，中间层 override 会压掉底层。
#[test]
fn three_level_extends_child_block_wins() {
    // base:     "<% block body %>BASE<% end %>"
    let base = tpl_ir(vec![block_node("body", vec![text_node("BASE")])]);
    // middle:   "<% extends 'base' %><% block body %>MIDDLE<% end %>"
    let middle = tpl_ir(vec![
        extends_node("base"),
        block_node("body", vec![text_node("MIDDLE")]),
    ]);
    // child:    "<% extends 'middle' %><% block body %>CHILD<% end %>"
    let child = tpl_ir(vec![
        extends_node("middle"),
        block_node("body", vec![text_node("CHILD")]),
    ]);

    let mut loader = InMemoryLoader::new()
        .insert("base", base)
        .insert("middle", middle)
        .insert("child", child);

    let entry = loader.resolve("child", None).unwrap().document;
    let out = render_ir_with_loader(
        &entry,
        &Value::Object(serde_json::Map::new()),
        &mut loader,
    ).unwrap();

    assert_eq!(out, "CHILD", "最底层 child 的 block override 必须穿透整条继承链");
}

/// 链式 include（main include a include b）：每层的 `from` 参数必须正确指向来源模板。
/// 修复前的 bug：include 后 `current_id` 不恢复，第二次 include 会按错误的来源解析。
#[test]
fn chained_include_restores_current_id() {
    // 用一个能记录 resolve 调用链的 loader，但只在渲染时记录（过滤入口 resolve）
    struct TraceLoader {
        docs: HashMap<String, IrDocument>,
        calls: Vec<(String, Option<String>)>, // (ref, from)
    }
    impl TemplateLoader for TraceLoader {
        fn resolve(&mut self, ref_: &str, from: Option<&str>) -> Result<ResolvedTemplate, IrError> {
            // 入口渲染时 resolve("main", None) 不算 include 调用链
            if from.is_some() || ref_ != "main" {
                self.calls.push((ref_.to_string(), from.map(|s| s.to_string())));
            }
            let doc = self.docs.get(ref_).cloned().ok_or_else(|| diag_load(codes::TEMPLATE_NOT_FOUND).arg("template", ref_))?;
            Ok(ResolvedTemplate { id: ref_.to_string(), document: doc })
        }
    }

    let main = tpl_ir(vec![include_node("a")]);
    let a = tpl_ir(vec![include_node("b")]);
    let b = tpl_ir(vec![text_node("B")]);

    let mut loader = TraceLoader {
        docs: HashMap::from([
            ("main".into(), main),
            ("a".into(), a),
            ("b".into(), b),
        ]),
        calls: vec![],
    };

    let entry = loader.resolve("main", None).unwrap().document;
    let out = render_ir_opts(
        &entry,
        &Value::Object(serde_json::Map::new()),
        RenderOptions { name: Some("main".into()), ..Default::default() },
        Some(&mut loader),
    ).unwrap();

    assert_eq!(out, "B");
    // 每次 include 的 from 必须正确：a 来自 main，b 来自 a（不是 main）
    assert_eq!(loader.calls, vec![
        ("a".into(), Some("main".into())),
        ("b".into(), Some("a".into())),
    ]);
}

/// include 带 extends 的模板：被 include 的模板自身有 extends 链，必须独立解析。
/// 修复前的 bug：render_loaded 直接调 render_nodes 而不是 render_doc，
/// 导致被 include 模板中的 Stmt.Extends 触发 EXTENDS_NOT_TOP_LEVEL。
#[test]
fn include_template_that_has_extends() {
    // base: "<% block body %>BASE<% end %>"
    let base = tpl_ir(vec![block_node("body", vec![text_node("BASE")])]);
    // child_of_base: "<% extends 'base' %><% block body %>INCLUDED_CHILD<% end %>"
    let child_of_base = tpl_ir(vec![
        extends_node("base"),
        block_node("body", vec![text_node("INCLUDED_CHILD")]),
    ]);
    // main: "<% include 'child_of_base' %>"
    let main = tpl_ir(vec![include_node("child_of_base")]);

    let mut loader = InMemoryLoader::new()
        .insert("base", base)
        .insert("child_of_base", child_of_base)
        .insert("main", main);

    let entry = loader.resolve("main", None).unwrap().document;
    let out = render_ir_with_loader(
        &entry,
        &Value::Object(serde_json::Map::new()),
        &mut loader,
    ).unwrap();

    // 被 include 的模板成功解析自己的 extends → body block → INCLUDED_CHILD
    assert_eq!(out, "INCLUDED_CHILD");
}

/// slug 的 NFKD 规范化：连字 `ﬃ` → `ffi`、全角 `ＡＢＣ` → `abc`。
/// 修复前用 NFD（canonical decomposition），不会分解连字和全角字符。
#[test]
fn slug_filter_nfkd_ligature_and_fullwidth() {
    let doc = tpl_ir(vec![interpolation_pipe("s", "slug", vec![])]);
    let mut ctx = serde_json::Map::new();
    // ffi 是拉丁连字 U+FB03，ＡＢＣ 是全角大写
    ctx.insert("s".into(), Value::String("ﬃ ＡＢＣ".into()));
    let out = render_ir_opts(
        &doc,
        &Value::Object(ctx),
        RenderOptions::default(),
        None,
    ).unwrap();
    // NFKD 把 ffi 连字分解成 ffi，全角大写分解成小写字母
    assert_eq!(out, "ffi-abc");
}

/// date 非法输入：`2024-99-99`（月/日越界）必须返回空串，对齐 TS `Date` Invalid 行为。
/// 修复前只 parse 数字不验证月/日范围，会原样输出 "2024-99-99"。
#[test]
fn date_filter_invalid_iso_returns_empty() {
    let doc = tpl_ir(vec![interpolation_pipe("d", "date", vec![])]);

    // 月越界
    let mut ctx = serde_json::Map::new();
    ctx.insert("d".into(), Value::String("2024-99-15".into()));
    let out = render_ir_opts(&doc, &Value::Object(ctx), RenderOptions::default(), None).unwrap();
    assert_eq!(out, "", "月份越界应返回空串");

    // 日越界
    let mut ctx = serde_json::Map::new();
    ctx.insert("d".into(), Value::String("2024-02-30".into())); // 2月没有30号
    let out = render_ir_opts(&doc, &Value::Object(ctx), RenderOptions::default(), None).unwrap();
    assert_eq!(out, "", "日期越界应返回空串");

    // 闰年边界：2024-02-29 合法
    let mut ctx = serde_json::Map::new();
    ctx.insert("d".into(), Value::String("2024-02-29".into()));
    let out = render_ir_opts(&doc, &Value::Object(ctx), RenderOptions::default(), None).unwrap();
    assert_eq!(out, "2024-02-29", "闰年2月29日应有效");

    // 平年：2023-02-29 非法
    let mut ctx = serde_json::Map::new();
    ctx.insert("d".into(), Value::String("2023-02-29".into()));
    let out = render_ir_opts(&doc, &Value::Object(ctx), RenderOptions::default(), None).unwrap();
    assert_eq!(out, "", "平年2月29日应返回空串");
}
