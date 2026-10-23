//! Single-template + catalog AOT codegen entry points.
//!
//! `precompile_single` produces one ESM module per template IR — no catalog
//! traversal. `precompile_catalog` accepts a catalog JSON blob and produces
//! a list of modules (entry + every template reachable via `include` /
//! `extends`) plus a manifest describing their relationship.

use std::collections::HashMap;

use dejavu_diagnostic::{Diagnostic, Severity, Stage};
use dejavu_ir::{IrDocument, IrNode, TemplateCatalog, canonical_id as ir_canonical_id};
use serde::Serialize;
use serde_json::Value;

use crate::catalog::{
    collect_all_reachable, collect_blocks, detect_cycle, js_import_name,
    parse_catalog_json, sanitize_id, split_canonical,
};
use crate::emit::{self, CatalogEmitCtx, CodegenCtx};
use crate::options::CodegenOptions;

// ---------------------------------------------------------------------------
// Result types
// ---------------------------------------------------------------------------

/// Result of a single-template precompile.
#[derive(Debug, Clone)]
pub struct PrecompileResult {
    /// Complete ESM module source, ready to `import { render } from "...";`.
    pub code: String,
    /// Imported template dependencies (includes / extends). Always empty for
    /// the single-template codegen path — catalog resolution fills this later.
    pub dependencies: Vec<String>,
    /// Structured diagnostics collected during compilation (Compile stage).
    pub diagnostics: Vec<Diagnostic>,
    /// Entry template id used for dependency tracking. Defaults to `"__entry__"`.
    pub entry_id: String,
}

/// One generated ESM module produced by catalog AOT.
#[derive(Debug, Clone, Serialize)]
pub struct GeneratedModule {
    /// Relative path inside the output root (e.g. `"index.js"`, `"_internal/layout__base.js"`).
    pub path: String,
    /// ESM JS source.
    pub code: String,
    /// Canonical ids of templates this module imports (via `include`).
    pub dependencies: Vec<String>,
}

/// Result of a catalog precompile.
#[derive(Debug, Clone)]
pub struct PrecompileCatalogResult {
    /// All generated modules. Entry first, then internal modules in an
    /// arbitrary but deterministic order (we iterate a `BTreeMap`).
    pub modules: Vec<GeneratedModule>,
    /// JSON manifest describing entry id, entry module path, and the
    /// canonical-id → file-path map for every module.
    pub manifest: String,
    /// All diagnostics collected (compile stage errors, cycle errors, ...).
    pub diagnostics: Vec<Diagnostic>,
    /// Canonical entry id as passed by the caller (or resolved default).
    pub entry_id: String,
}

// ---------------------------------------------------------------------------
// Module manifest (serialized as JSON)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct Manifest<'a> {
    format: &'a str,
    entry_id: &'a str,
    entry_module: &'a str,
    modules: &'a serde_json::Map<String, Value>,
    version: u32,
}

// ---------------------------------------------------------------------------
// Single-template path
// ---------------------------------------------------------------------------

/// Import path for the runtime helpers.
pub(crate) const RUNTIME_IMPORT: &str = "@doki-land/dejavu-engine/runtime";

/// Top-level single-template entry point.
///
/// - `ir_json` — IR document JSON (as emitted by dejavu-ir / dejavu-language).
/// - `options_json` — optional JSON string with codegen knobs.
pub fn precompile_single(
    ir_json: &str,
    options_json: Option<&str>,
) -> Result<PrecompileResult, String> {
    let doc: IrDocument = serde_json::from_str(ir_json)
        .map_err(|e| format!("invalid IR JSON: {e}"))?;

    let opts = CodegenOptions::from_json(options_json)
        .map_err(|e| format!("invalid options JSON: {e}"))?;

    let entry_id = opts.entry_id().to_string();
    let mut ctx = CodegenCtx::new(&opts);

    let body_str = match &doc.body {
        IrNode::Template { children, .. } => {
            let mut out = String::new();
            emit::emit_body(children, &mut ctx, &mut out, None);
            out
        }
        other => {
            ctx.diagnostic(
                Severity::Error,
                "dejavu.compile.root-not-template",
                format!("IR root node must be Template, got {other:?}"),
            );
            String::new()
        }
    };

    let code = build_module(&body_str, &[], None);

    Ok(PrecompileResult {
        code,
        dependencies: Vec::new(),
        diagnostics: ctx.diagnostics,
        entry_id,
    })
}

// ---------------------------------------------------------------------------
// Catalog path
// ---------------------------------------------------------------------------

/// Top-level catalog AOT entry point.
///
/// - `entry_ir_json` — ignored in the current shape but kept for API parity
///   with `precompile_single`; the entry is resolved from `options.entryId`
///   (default: the first root's first file), or from the catalog's files.
/// - `catalog_json` — the full catalog wire format (roots[].files[path] = IR JSON).
/// - `options_json` — optional codegen knobs.
pub fn precompile_catalog(
    _entry_ir_json: &str,
    catalog_json: &str,
    options_json: Option<&str>,
) -> Result<PrecompileCatalogResult, String> {
    let catalog = parse_catalog_json(catalog_json)?;
    let opts = CodegenOptions::from_json(options_json)
        .map_err(|e| format!("invalid options JSON: {e}"))?;

    // Pick entry id. options.entryId wins; otherwise first file in first root.
    let entry_id = pick_entry_id(&catalog, opts.entry_id.as_deref());

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // 1. Cycle detection on include/extends edges.
    if let Err(diag) = detect_cycle(&catalog, &entry_id) {
        diagnostics.push(diag);
        return Ok(PrecompileCatalogResult {
            modules: Vec::new(),
            manifest: String::new(),
            diagnostics,
            entry_id,
        });
    }

    // 2. Collect the set of reachable template ids (include + extends closure).
    let reachable = collect_all_reachable(&catalog, &entry_id);

    // 3. Module naming plan: canonical id → file path.
    let mut id_to_path: serde_json::Map<String, Value> = serde_json::Map::new();
    let mut id_to_deps: HashMap<String, Vec<String>> = HashMap::new();

    // First pass — decide every module's path.
    for id in &reachable {
        if id == &entry_id {
            // Entry → always "index.js" by default. Callers can customise via
            // an `entryFile` option later; for now we keep it fixed.
            id_to_path.insert(id.clone(), Value::String("index.js".into()));
        } else {
            let file_name = format!("_internal/{}.js", sanitize_id(id));
            id_to_path.insert(id.clone(), Value::String(file_name));
        }
    }

    // 4. Emit each reachable template.
    let mut modules: Vec<GeneratedModule> = Vec::new();
    let mut id_to_module: HashMap<String, GeneratedModule> = HashMap::new();

    for id in &reachable {
        let Some(doc) = catalog_document(&catalog, id) else {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Error,
                    "dejavu.compile.template-missing",
                    Stage::Compile,
                )
                .template_id(id)
                .message(format!("template {id:?} referenced by catalog but not found")),
            );
            continue;
        };

        // Collect merged blocks + parent blocks for this template.
        let (merged_blocks, topmost_parent_id) = match collect_blocks(&catalog, id) {
            Ok(pair) => pair,
            Err(d) => {
                diagnostics.push(d);
                continue;
            }
        };
        let parent_blocks = match crate::catalog::collect_parent_blocks(&catalog, id) {
            Ok(m) => m,
            Err(d) => {
                diagnostics.push(d);
                continue;
            }
        };

        // Walk the document for include refs (pre-resolve to canonical id).
        let include_imports = resolve_include_imports(&catalog, doc, id);

        // Collect include dependencies (canonical ids of templates this
        // module directly imports).
        let mut dependencies: Vec<String> = include_imports.values().cloned().collect();
        dependencies.sort();
        dependencies.dedup();
        id_to_deps.insert(id.clone(), dependencies.clone());

        // Render body comes from topmost parent's document (entry is flattened).
        let render_doc = if id == &entry_id {
            // Entry: flattened body from the topmost parent.
            let Some(doc) = catalog_document(&catalog, &topmost_parent_id) else {
                diagnostics.push(
                    Diagnostic::new(
                        Severity::Error,
                        "dejavu.compile.parent-template-missing",
                        Stage::Compile,
                    )
                    .template_id(&topmost_parent_id),
                );
                continue;
            };
            doc
        } else {
            doc
        };

        // If this is an entry template that extends, the render body is the
        // topmost parent's body — strip any Stmt.Extends that might be there
        // (they should be top-level only).
        let body_children = match &render_doc.body {
            IrNode::Template { children, .. } => children.clone(),
            _ => Vec::new(),
        };

        let cat_ctx = CatalogEmitCtx {
            merged_blocks: &merged_blocks,
            parent_blocks: &parent_blocks,
            include_imports: &include_imports,
        };

        let mut ctx = CodegenCtx::new(&opts);
        let mut body_str = String::new();
        emit::emit_body(&body_children, &mut ctx, &mut body_str, Some(&cat_ctx));

        // Drop Stmt.Extends nodes from the emitted body — they have already
        // been flattened. The emit_stmt above will still append a warning
        // diagnostic if any are found, but we keep the module parseable.
        // Actually, the body_children we took from the topmost parent won't
        // have Stmt.Extends — only the child would. Good.

        let entry_imports = build_import_lines(&dependencies, id, &id_to_path);
        let code = build_module(&body_str, &entry_imports, Some(&id_to_deps));

        diagnostics.extend(ctx.diagnostics);

        let path = id_to_path.get(id).unwrap().as_str().unwrap_or("").to_string();
        let m = GeneratedModule { path, code, dependencies };
        id_to_module.insert(id.clone(), m.clone());
        modules.push(m);
    }

    // Sort entry first, then alphabetical by path for determinism.
    let entry_path = id_to_path
        .get(&entry_id)
        .and_then(|v| v.as_str())
        .unwrap_or("index.js")
        .to_string();
    modules.sort_by(|a, b| {
        let a_entry = a.path == entry_path;
        let b_entry = b.path == entry_path;
        match (a_entry, b_entry) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.path.cmp(&b.path),
        }
    });

    // 5. Build manifest.
    let manifest = build_manifest(&entry_id, &id_to_path);

    Ok(PrecompileCatalogResult {
        modules,
        manifest,
        diagnostics,
        entry_id,
    })
}

// ---------------------------------------------------------------------------
// Helpers: module rendering
// ---------------------------------------------------------------------------

/// Wrap raw body fragments into a full ESM module, prepending any template
/// `import` statements *after* the runtime import.
pub(crate) fn build_module(body: &str, template_imports: &[String], _dep_map: Option<&HashMap<String, Vec<String>>>) -> String {
    let mut s = String::with_capacity(512 + body.len());
    s.push_str(&format!(
        "import {{ htmlEscape, applyFilter, isTruthy, binary, unary, valueToString, exprMember, exprIndex }} from \"{RUNTIME_IMPORT}\";\n"
    ));
    for imp in template_imports {
        s.push_str(imp);
        s.push('\n');
    }
    s.push('\n');
    s.push_str("export function render(ctx, options = {}) {\n");
    s.push_str("  if (!ctx || typeof ctx !== 'object') ctx = {};\n");
    s.push_str("  const __strict = options.strictUndefined === true;\n");
    s.push_str("  const __out = [];\n");
    s.push('\n');
    for line in body.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            s.push('\n');
        } else {
            s.push_str("  ");
            s.push_str(trimmed);
            s.push('\n');
        }
    }
    s.push_str("  return __out.join(\"\");\n");
    s.push_str("}\n");
    s
}

/// Build ESM import statements for a module given its dependencies and
/// the global id → path map.
fn build_import_lines(
    dependencies: &[String],
    current_id: &str,
    id_to_path: &serde_json::Map<String, Value>,
) -> Vec<String> {
    let mut out = Vec::new();
    let current_path = match id_to_path.get(current_id) {
        Some(Value::String(p)) => p.as_str(),
        _ => "",
    };
    for dep_id in dependencies {
        let Some(Value::String(target_path)) = id_to_path.get(dep_id) else { continue; };
        let binding = js_import_name(dep_id);
        let rel = relative_import_path(current_path, target_path);
        out.push(format!("import {{ {binding} }} from \"{rel}\";"));
    }
    out
}

/// Compute a POSIX relative ESM import path from `from` to `to`.
///
/// Both paths are relative to an (implicit) output root. Uses `..` if they
/// share no common ancestor. `from` is treated as a *file* path so we strip
/// its last segment before computing the relative directory.
fn relative_import_path(from: &str, to: &str) -> String {
    let from_dir = match from.rfind('/') {
        Some(i) => &from[..i],
        None => "",
    };
    let to_dir = match to.rfind('/') {
        Some(i) => &to[..i],
        None => "",
    };
    let to_file = match to.rfind('/') {
        Some(i) => &to[i + 1..],
        None => to,
    };

    // Strip common prefix.
    let from_parts: Vec<&str> = from_dir.split('/').filter(|s| !s.is_empty()).collect();
    let to_parts: Vec<&str> = to_dir.split('/').filter(|s| !s.is_empty()).collect();

    let mut common = 0;
    while common < from_parts.len() && common < to_parts.len() && from_parts[common] == to_parts[common] {
        common += 1;
    }
    let up_count = from_parts.len() - common;
    let mut rel = if up_count == 0 {
        let remaining_to = to_parts[common..].join("/");
        if remaining_to.is_empty() { format!("./{to_file}") } else { format!("./{}/{to_file}", remaining_to) }
    } else {
        let prefix = "../".repeat(up_count);
        let remaining_to = to_parts[common..].join("/");
        if remaining_to.is_empty() { format!("{prefix}{to_file}") } else { format!("{prefix}{remaining_to}/{to_file}") }
    };
    // Ensure starts with "./" or "../".
    if !rel.starts_with("./") && !rel.starts_with("../") {
        rel = format!("./{rel}");
    }
    rel
}

// ---------------------------------------------------------------------------
// Catalog helpers
// ---------------------------------------------------------------------------

fn pick_entry_id(catalog: &TemplateCatalog, explicit: Option<&str>) -> String {
    if let Some(id) = explicit {
        return id.to_string();
    }
    // First file in first root becomes the implicit entry.
    catalog.roots.iter()
        .find_map(|r| {
            r.files.keys().next().map(|p| ir_canonical_id(&r.name, p))
        })
        .unwrap_or_else(|| "__entry__".to_string())
}

fn catalog_document<'a>(catalog: &'a TemplateCatalog, id: &str) -> Option<&'a IrDocument> {
    let (root_name, path) = split_canonical(id)?;
    for r in &catalog.roots {
        if r.name == root_name {
            return r.files.get(path);
        }
    }
    None
}

/// Walk document children collecting every `Stmt.Include` path literal
/// and resolving it to a canonical id.
fn resolve_include_imports(
    catalog: &TemplateCatalog,
    doc: &IrDocument,
    from_id: &str,
) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let body = match &doc.body {
        IrNode::Template { children, .. } => children,
        _ => return out,
    };
    collect_includes_from_nodes(catalog, body, from_id, &mut out);
    out
}

fn collect_includes_from_nodes(
    catalog: &TemplateCatalog,
    nodes: &[IrNode],
    from_id: &str,
    out: &mut HashMap<String, String>,
) {
    for n in nodes {
        match n {
            IrNode::StmtInclude { path, .. } => {
                if let Some(lit) = crate::catalog::extract_path_literal(path) {
                    if let Ok(hit) = catalog.resolve_ref(&lit, Some(from_id)) {
                        if let Some(canon) = catalog.canonical_id(&hit) {
                            out.insert(lit, canon);
                        }
                    }
                }
            }
            IrNode::StmtIf { consequent, else_ifs, alternate, .. } => {
                collect_includes_from_nodes(catalog, consequent, from_id, out);
                for ei in else_ifs {
                    if let IrNode::StmtElseIf { consequent, .. } = ei {
                        collect_includes_from_nodes(catalog, consequent, from_id, out);
                    }
                }
                if let Some(alt) = alternate {
                    collect_includes_from_nodes(catalog, alt, from_id, out);
                }
            }
            IrNode::StmtFor { body, .. } => {
                collect_includes_from_nodes(catalog, body, from_id, out);
            }
            IrNode::Template { children, .. } => {
                collect_includes_from_nodes(catalog, children, from_id, out);
            }
            _ => {}
        }
    }
}

/// Build the manifest JSON string.
fn build_manifest(entry_id: &str, id_to_path: &serde_json::Map<String, Value>) -> String {
    let entry_module = id_to_path
        .get(entry_id)
        .and_then(|v| v.as_str())
        .unwrap_or("index.js")
        .to_string();
    let manifest = Manifest {
        format: "dejavu.aot.catalog/1",
        entry_id,
        entry_module: &entry_module,
        modules: id_to_path,
        version: 1,
    };
    serde_json::to_string(&manifest).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Public helpers / re-exports
// ---------------------------------------------------------------------------

/// Parse IR JSON to an IrDocument (public for tests / future crates).
pub fn parse_ir_json(json: &str) -> Result<IrDocument, String> {
    serde_json::from_str(json).map_err(|e| format!("invalid IR JSON: {e}"))
}

/// Parse options JSON to a Value (public for callers who want extra knobs later).
pub fn parse_options_json(json: Option<&str>) -> Option<Value> {
    json.and_then(|j| serde_json::from_str::<Value>(j).ok())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_import_path_same_dir() {
        assert_eq!(relative_import_path("index.js", "_internal/a.js"), "./_internal/a.js");
        assert_eq!(relative_import_path("_internal/a.js", "_internal/b.js"), "./b.js");
    }

    #[test]
    fn relative_import_path_parent_dir() {
        assert_eq!(relative_import_path("_internal/a.js", "index.js"), "../index.js");
    }

    #[test]
    fn relative_import_path_shared_ancestor() {
        assert_eq!(
            relative_import_path("_internal/shared/a.js", "_internal/other/b.js"),
            "../other/b.js"
        );
    }

    #[test]
    fn relative_import_path_deep() {
        // from dir has 3 segments, to dir has 2 → up 2 levels.
        assert_eq!(
            relative_import_path("_internal/layout/deep/a.js", "_internal/partial/b.js"),
            "../../partial/b.js"
        );
    }
}
