//! Render a multi-template catalog fixture (inheritance / loader conformance).
//!
//! Usage: render_catalog <fixture_dir>
//!
//! `<fixture_dir>` 必须包含：
//!   `entry.txt`          — 入口模板的 canonical path（如 `child.dejavu`）
//!   `templates/*`        — 一个或多个模板源码文件
//!   `context.ctx.json`   — JSON context
//!   `language.json`       — 可选，覆盖默认 delimiters（`<% %>` / `<# #>`）
//!
//! 脚本会 parse 所有模板 → 构建 TemplateCatalog → CatalogLoader →
//! 然后走 `render_ir_opts` 渲染入口。stdout 就是渲染输出。

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

use dejavu_ir::{
    CatalogRoot, Language, ResolveHit, RenderOptions, ResolvedTemplate, TemplateCatalog, TemplateLoader,
    diag_load, default_language, render_ir_opts,
};
use dejavu_language::parse_with as dejavu_parse_with;

fn main() {
    let fixture_dir = env::args().nth(1).expect("usage: render_catalog <fixture_dir>");
    let fixture = Path::new(&fixture_dir);
    let templates_dir = fixture.join("templates");
    let ctx_path = fixture.join("context.ctx.json");
    let entry_path = fixture.join("entry.txt");
    let language_path = fixture.join("language.json");

    // ---- 读 context ----
    let ctx_str = fs::read_to_string(&ctx_path).expect("read context.ctx.json");
    let ctx: serde_json::Value = serde_json::from_str(&ctx_str).expect("parse context JSON");

    // ---- 读 entry ----
    let entry = fs::read_to_string(&entry_path)
        .expect("read entry.txt")
        .trim()
        .to_string();

    // ---- 读可选的 language.json ----
    let language: Language = if language_path.exists() {
        let lang_str = fs::read_to_string(&language_path).expect("read language.json");
        serde_json::from_str(&lang_str).expect("parse language.json")
    } else {
        default_language()
    };

    // ---- 遍历 templates/*，parse 并登记 ----
    let mut files = BTreeMap::<String, _>::new();
    for entry in fs::read_dir(&templates_dir).expect("read templates dir") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if !path.is_file() { continue; }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let source = fs::read_to_string(&path).expect(&format!("read {}", name));
        let ir = match dejavu_parse_with(&source, &name, &language) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("SKIP {name}: parser does not support this template syntax ({e})");
                std::process::exit(0);
            }
        };
        files.insert(name, ir);
    }

    // ---- 构建 catalog ----
    let mut root = CatalogRoot::new("fixture", "fixture", 10);
    for (name, ir) in files {
        root.insert(&name, ir);
    }
    let catalog = TemplateCatalog::new(vec![root], vec![".dejavu".into(), "".into()]);

    // ---- render ----
    let mut loader = FixtureLoader { catalog: &catalog };
    let Some(entry_doc) = catalog.document(
        &ResolveHit { root_index: 0, path: entry.clone() }
    ) else {
        eprintln!("entry template `{entry}` not found in catalog");
        std::process::exit(2);
    };

    let out = render_ir_opts(
        entry_doc,
        &ctx,
        RenderOptions { name: Some(entry), ..Default::default() },
        Some(&mut loader),
    ).expect("render");

    print!("{out}");
}

/// 把 TemplateCatalog 包装成 TemplateLoader。
struct FixtureLoader<'a> {
    catalog: &'a TemplateCatalog,
}

impl TemplateLoader for FixtureLoader<'_> {
    fn resolve(&mut self, ref_: &str, from: Option<&str>) -> Result<ResolvedTemplate, dejavu_ir::IrError> {
        let hit = self.catalog.resolve_ref(ref_, from).map_err(Into::<dejavu_ir::IrError>::into)?;
        let Some(doc) = self.catalog.document(&hit) else {
            return Err(diag_load(dejavu_ir::codes::TEMPLATE_NOT_FOUND).arg("template", ref_));
        };
        let canonical = self.catalog.canonical_id(&hit).unwrap_or_else(|| ref_.to_string());
        Ok(ResolvedTemplate { id: canonical, document: doc.clone() })
    }
}
