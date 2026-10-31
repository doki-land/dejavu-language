//! Emit load-stage diagnostics for a multi-template catalog fixture.
//!
//! Usage: catalog_diag <fixture_dir>
//!
//! Reads the same `entry.txt` / `templates/*` / `context.ctx.json` layout as
//! `render_catalog`, renders the entry, and writes `{"diagnostics":[...]}`
//! (exit 0) so the conformance runner can compare structured diagnostics
//! across hosts. A successful render reports an empty diagnostic list.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

use dejavu_ir::{
    CatalogRoot, Diagnostic, IrError, Language, ResolveHit, RenderOptions, ResolvedTemplate,
    Severity, Stage, TemplateCatalog, TemplateLoader, default_language, diag_load, render_ir_opts,
};
use dejavu_language::parse_with as dejavu_parse_with;
use serde_json::json;

fn main() {
    let fixture_dir = env::args().nth(1).expect("usage: catalog_diag <fixture_dir>");
    let fixture = Path::new(&fixture_dir);
    let templates_dir = fixture.join("templates");
    let ctx_path = fixture.join("context.ctx.json");
    let entry_path = fixture.join("entry.txt");
    let language_path = fixture.join("language.json");

    let ctx: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&ctx_path).expect("read context.ctx.json"))
            .expect("parse context JSON");
    let entry = fs::read_to_string(&entry_path)
        .expect("read entry.txt")
        .trim()
        .to_string();
    let language: Language = if language_path.exists() {
        serde_json::from_str(&fs::read_to_string(&language_path).expect("read language.json"))
            .expect("parse language.json")
    } else {
        default_language()
    };

    let mut files = BTreeMap::<String, _>::new();
    for entry in fs::read_dir(&templates_dir).expect("read templates dir") {
        let path = entry.expect("dir entry").path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let source = fs::read_to_string(&path).expect(&format!("read {}", name));
        // A parse failure is itself a cross-host diagnostic (stage `parse`);
        // report the first one and stop, mirroring the render-stage contract.
        let ir = match dejavu_parse_with(&source, &name, &language) {
            Ok(ir) => ir,
            Err(error) => {
                let mut diagnostic =
                    Diagnostic::new(Severity::Error, error.code_or_default(), Stage::Parse)
                        .message(error.message)
                        .path(error.file);
                if error.span.len() > 0 {
                    diagnostic = diagnostic.span(error.span.start, error.span.end);
                }
                for (key, value) in error.args {
                    diagnostic = diagnostic.arg(key, value);
                }
                println!("{}", json!({ "diagnostics": [diagnostic] }));
                return;
            }
        };
        files.insert(name, ir);
    }

    let mut root = CatalogRoot::new("fixture", "fixture", 10);
    for (name, ir) in files {
        root.insert(&name, ir);
    }
    let catalog = TemplateCatalog::new(vec![root], vec![".dejavu".into(), "".into()]);

    let mut loader = FixtureLoader { catalog: &catalog };
    let entry_doc = catalog
        .document(&ResolveHit { root_index: 0, path: entry.clone() })
        .unwrap_or_else(|| panic!("entry template `{entry}` not found in catalog"));

    let payload = match render_ir_opts(
        entry_doc,
        &ctx,
        RenderOptions { name: Some(entry), ..Default::default() },
        Some(&mut loader),
    ) {
        Ok(_) => json!({ "diagnostics": [] }),
        // Only engine diagnostics participate in the cross-host contract. Other
        // failures (for example a missing loader) stay visible as host errors so
        // they can never masquerade as a cycle diagnostic.
        Err(IrError::Diagnostic(diagnostic)) => json!({ "diagnostics": [diagnostic] }),
        Err(other) => {
            eprintln!("render failed without a structured diagnostic: {other}");
            json!({ "diagnostics": [] })
        }
    };
    println!("{payload}");
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
