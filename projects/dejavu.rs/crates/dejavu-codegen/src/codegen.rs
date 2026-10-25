//! Single-template + catalog AOT codegen entry points.
//!
//! `precompile_single` produces one ESM module per template IR — no catalog
//! traversal. `precompile_catalog` accepts a catalog JSON blob and produces
//! a list of modules (entry + every template reachable via `include` /
//! `extends`) plus a manifest describing their relationship.

use std::collections::{HashMap, HashSet};

use dejavu_diagnostic::{codes, Diagnostic, Severity, Stage};
use dejavu_ir::{IrDocument, IrNode, TemplateCatalog, canonical_id as ir_canonical_id};
use serde::Serialize;
use serde_json::Value;

use crate::catalog::{
    collect_all_reachable, collect_blocks, detect_cycle, js_import_name,
    parse_catalog_json, sanitize_id, split_canonical,
};
use crate::emit::{self, CatalogEmitCtx, CodegenCtx, LineOrigin};
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
    /// Generated-line → template-span points for source maps.
    pub origins: Vec<LineOrigin>,
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
    /// Generated-line → template-span points for source maps.
    pub origins: Vec<LineOrigin>,
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

/// Everything catalog traversal needs to know about one template before it can
/// be emitted: its flattened render body, block chains, include bindings, and
/// resolved macro imports.
struct ModulePlan {
    /// Canonical template id.
    id: String,
    /// Body actually emitted: the topmost parent for the entry, the template's
    /// own children otherwise. A `partial` template contributes its
    /// declaration only.
    body_children: Vec<IrNode>,
    /// Every ancestor definition of each block, most-derived first.
    block_chains: HashMap<String, Vec<Vec<IrNode>>>,
    /// Include path literal → canonical target id.
    include_imports: HashMap<String, String>,
    /// Canonical ids of templates this module includes.
    dependencies: Vec<String>,
    /// Resolved `from ... import` bindings.
    macro_imports: Vec<MacroImport>,
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

    let mut body_str = String::new();
    let mut macro_fns = String::new();
    match &doc.body {
        IrNode::Template { children, .. } => {
            ctx.set_macros(macro_names(children));
            emit::emit_body(children, &mut ctx, &mut body_str, None);
            macro_fns = emit::emit_macro_functions(children, &mut ctx, None);
        }
        other => ctx.diagnostic(
            Severity::Error,
            "dejavu.compile.root-not-template",
            format!("IR root node must be Template, got {other:?}"),
        ),
    }

    let code = build_module(&body_str, &macro_fns, &[], &[], None, opts.strict_undefined, None);
    let origins = shift_origins(&ctx.origins, module_header_lines(0, macro_lines(&macro_fns)));

    Ok(PrecompileResult {
        code,
        dependencies: Vec::new(),
        diagnostics: ctx.diagnostics,
        entry_id,
        origins,
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

    // 2b. Which reachable templates declare `partial <params>`?
    let mut partial_ids: HashSet<String> = HashSet::new();
    for id in &reachable {
        if crate::catalog::is_partial_template(&catalog, id) {
            partial_ids.insert(id.clone());
        }
    }

    // 3. Module naming plan: canonical id → file path.
    let mut id_to_path: serde_json::Map<String, Value> = serde_json::Map::new();

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

    // 4. Resolve each module's emitted body, block chains, and macro imports.
    let plans = build_module_plans(&catalog, &reachable, &entry_id, &partial_ids, &mut diagnostics);
    let id_to_deps: HashMap<String, Vec<String>> =
        plans.iter().map(|plan| (plan.id.clone(), plan.dependencies.clone())).collect();

    // 5. Macro tree-shaking: only declarations reachable from a real call site
    //    (direct calls, `call` blocks, or calls inside live macros) are kept.
    //    A macro no live body can invoke has no observable effect.
    let live_macros = reachable_macros(&plans);

    // 6. Emit each module with only its live macros and used imports.
    let mut modules: Vec<GeneratedModule> = Vec::new();

    for plan in &plans {
        let id = &plan.id;

        // Declaration order is preserved so generated modules stay stable.
        let mut live_names: Vec<String> = macro_names(&plan.body_children);
        live_names.retain(|name| live_macros.contains(&(id.clone(), name.clone())));
        let live_set: HashSet<&str> = live_names.iter().map(String::as_str).collect();

        let used_names = used_call_names(plan, &live_macros);
        let used_imports: Vec<MacroImport> = plan
            .macro_imports
            .iter()
            .filter(|imported| used_names.contains(&imported.local))
            .cloned()
            .collect();
        let macro_nodes: Vec<IrNode> = plan
            .body_children
            .iter()
            .filter(|node| matches!(node, IrNode::StmtMacro { name, .. } if live_set.contains(name.as_str())))
            .cloned()
            .collect();

        let cat_ctx = CatalogEmitCtx {
            block_chains: &plan.block_chains,
            include_imports: &plan.include_imports,
            partial_ids: &partial_ids,
        };

        let mut ctx = CodegenCtx::new(&opts);
        ctx.set_macros(live_names.iter().cloned());
        for imported in &used_imports {
            ctx.set_macro_ref(&imported.local, &emit::macro_import_binding(&imported.local));
        }
        let mut body_str = String::new();
        emit::emit_body(&plan.body_children, &mut ctx, &mut body_str, Some(&cat_ctx));
        let macro_fns = emit::emit_macro_functions(&macro_nodes, &mut ctx, Some(&cat_ctx));

        let entry_imports = build_import_lines(&plan.dependencies, id, &id_to_path);
        let macro_import_lines = build_macro_import_lines(&used_imports, id, &id_to_path);
        let mut template_imports = entry_imports;
        template_imports.extend(macro_import_lines);
        let exports = macro_export_lines(&live_names, &used_imports);
        let export_name = js_import_name(id);
        let code = build_module(
            &body_str,
            &macro_fns,
            &template_imports,
            &exports,
            Some(&id_to_deps),
            opts.strict_undefined,
            Some(&export_name),
        );
        let origins = shift_origins(
            &ctx.origins,
            module_header_lines(template_imports.len(), macro_lines(&macro_fns)),
        );

        diagnostics.extend(ctx.diagnostics);

        let path = id_to_path.get(id).unwrap().as_str().unwrap_or("").to_string();
        modules.push(GeneratedModule {
            path,
            code,
            dependencies: plan.dependencies.clone(),
            origins,
        });
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
// Helpers: macro tree-shaking
// ---------------------------------------------------------------------------

/// `(declaring module, macro name)` — the unit of macro reachability.
type MacroKey = (String, String);

/// Resolve every reachable template into an emit-ready [`ModulePlan`].
///
/// This is a separate pass from emission so macro reachability can be computed
/// before any module decides which declarations to generate.
fn build_module_plans(
    catalog: &TemplateCatalog,
    reachable: &HashSet<String>,
    entry_id: &str,
    partial_ids: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ModulePlan> {
    let mut plans = Vec::new();
    for id in reachable {
        let Some(doc) = catalog_document(catalog, id) else {
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

        // The entry body is flattened from the topmost parent; blocks are
        // emitted through their per-block ancestor chain (`super` support).
        let topmost_parent_id = match collect_blocks(catalog, id) {
            Ok(topmost) => topmost,
            Err(d) => {
                diagnostics.push(d);
                continue;
            }
        };
        let block_chains = match crate::catalog::collect_block_chains(catalog, id) {
            Ok(chains) => chains,
            Err(d) => {
                diagnostics.push(d);
                continue;
            }
        };

        let include_imports = resolve_include_imports(catalog, doc, id);
        let mut dependencies: Vec<String> = include_imports.values().cloned().collect();
        dependencies.sort();
        dependencies.dedup();

        let render_doc = if id == entry_id {
            let Some(parent) = catalog_document(catalog, &topmost_parent_id) else {
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
            parent
        } else {
            doc
        };

        let body_children = match &render_doc.body {
            IrNode::Template { children, .. } => {
                if partial_ids.contains(id) {
                    // A partial template may only carry whitespace/comments
                    // outside its declaration; emit just the declaration.
                    children
                        .iter()
                        .filter(|n| matches!(n, IrNode::StmtPartial { .. }))
                        .cloned()
                        .collect()
                } else {
                    children.clone()
                }
            }
            _ => Vec::new(),
        };

        let macro_imports = resolve_macro_imports(catalog, &body_children, id, diagnostics);

        plans.push(ModulePlan {
            id: id.clone(),
            body_children,
            block_chains,
            include_imports,
            dependencies,
            macro_imports,
        });
    }
    plans
}

/// A macro declaration is live when some emitted body can invoke it.
///
/// Seeds come from every module's emitted render body and block chains. They
/// expand through macro bodies: a live macro exposes the calls it makes inside
/// its own declaring module's macro table (local declarations plus that
/// module's `from ... import` bindings), matching how the renderer resolves
/// macro calls.
fn reachable_macros(plans: &[ModulePlan]) -> HashSet<MacroKey> {
    let by_id: HashMap<&str, &ModulePlan> =
        plans.iter().map(|plan| (plan.id.as_str(), plan)).collect();
    let local: HashMap<&str, HashSet<String>> = plans
        .iter()
        .map(|plan| (plan.id.as_str(), macro_names(&plan.body_children).into_iter().collect()))
        .collect();
    let imports: HashMap<&str, HashMap<String, MacroKey>> = plans
        .iter()
        .map(|plan| {
            let table = plan
                .macro_imports
                .iter()
                .map(|imported| {
                    (
                        imported.local.clone(),
                        (imported.dep_id.clone(), imported.imported.clone()),
                    )
                })
                .collect();
            (plan.id.as_str(), table)
        })
        .collect();

    let resolve = |from: &str, name: &str| -> Option<MacroKey> {
        if local.get(from).is_some_and(|names| names.contains(name)) {
            return Some((from.to_string(), name.to_string()));
        }
        imports.get(from).and_then(|table| table.get(name)).cloned()
    };

    let mut live: HashSet<MacroKey> = HashSet::new();
    let mut worklist: Vec<MacroKey> = Vec::new();

    // Seed: every body that a module actually renders (its own body plus the
    // block chain it emits, covering `super` levels).
    for plan in plans {
        let mut names = HashSet::new();
        crate::catalog::collect_macro_calls(&plan.body_children, true, &mut names);
        for bodies in plan.block_chains.values() {
            for body in bodies {
                crate::catalog::collect_macro_calls(body, true, &mut names);
            }
        }
        for name in names {
            if let Some(key) = resolve(&plan.id, &name) {
                if live.insert(key.clone()) {
                    worklist.push(key);
                }
            }
        }
    }

    // Expand through live macro bodies.
    while let Some(key) = worklist.pop() {
        let Some(plan) = by_id.get(key.0.as_str()) else { continue };
        let Some(node) = macro_node(&plan.body_children, &key.1) else { continue };
        let mut names = HashSet::new();
        crate::catalog::collect_macro_calls(std::slice::from_ref(node), false, &mut names);
        for name in names {
            if let Some(next) = resolve(&plan.id, &name) {
                if live.insert(next.clone()) {
                    worklist.push(next);
                }
            }
        }
    }

    live
}

/// Every macro-call candidate name emitted by this module, including calls
/// inside its live macro bodies.
fn used_call_names(plan: &ModulePlan, live: &HashSet<MacroKey>) -> HashSet<String> {
    let mut names = HashSet::new();
    crate::catalog::collect_macro_calls(&plan.body_children, true, &mut names);
    for bodies in plan.block_chains.values() {
        for body in bodies {
            crate::catalog::collect_macro_calls(body, true, &mut names);
        }
    }
    for name in macro_names(&plan.body_children) {
        if !live.contains(&(plan.id.clone(), name.clone())) {
            continue;
        }
        if let Some(node) = macro_node(&plan.body_children, &name) {
            crate::catalog::collect_macro_calls(std::slice::from_ref(node), false, &mut names);
        }
    }
    names
}

/// The top-level macro declaration named `name`, if any.
fn macro_node<'a>(children: &'a [IrNode], name: &str) -> Option<&'a IrNode> {
    children
        .iter()
        .find(|node| matches!(node, IrNode::StmtMacro { name: declared, .. } if declared == name))
}

// ---------------------------------------------------------------------------
// Helpers: module rendering
// ---------------------------------------------------------------------------

/// Wrap raw body fragments into a full ESM module, prepending any template
/// `import` statements *after* the runtime import.
pub(crate) fn build_module(
    body: &str,
    macro_fns: &str,
    template_imports: &[String],
    macro_exports: &[String],
    _dep_map: Option<&HashMap<String, Vec<String>>>,
    strict_undefined: bool,
    export_name: Option<&str>,
) -> String {
    let mut s = String::with_capacity(512 + body.len() + macro_fns.len());
    s.push_str(&format!(
        "import {{ htmlEscape, applyFilter, isTruthy, binary, unary, valueToString, exprMember, exprIndex, callFunction, applyTest, runtimeError }} from \"{RUNTIME_IMPORT}\";\n"
    ));
    for imp in template_imports {
        s.push_str(imp);
        s.push('\n');
    }
    s.push('\n');
    s.push_str(macro_fns);
    s.push_str("export function render(ctx, options = {}, __partialDepth = 0) {\n");
    s.push_str("  if (!ctx || typeof ctx !== 'object') ctx = {};\n");
    // Build-time `strictUndefined` is the default; an explicit per-call value
    // overrides it.
    s.push_str(&format!(
        "  const __strict = options.strictUndefined === undefined ? {strict_undefined} : options.strictUndefined === true;\n"
    ));
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
    // Included templates are imported by their canonical-id binding; keep the
    // stable `render` export and alias it for cross-module includes.
    if let Some(name) = export_name {
        s.push_str(&format!("\nexport const {name} = render;\n"));
    }
    // Macro functions stay importable across modules (`from ... import`).
    for line in macro_exports {
        s.push_str(line);
        s.push('\n');
    }
    s
}

/// Number of lines [`build_module`] writes before the first body line.
///
/// Body lines map 1:1 onto module lines (empty body lines still occupy a line),
/// so this offset is all a source map needs.
fn module_header_lines(template_imports: usize, macro_lines: u32) -> u32 {
    // runtime import + template imports, blank, macro functions,
    // `export function render`, three prologue statements, blank.
    u32::try_from(1 + template_imports + 1 + 4 + 1)
        .unwrap_or(u32::MAX)
        .saturating_add(macro_lines)
}

/// Lines occupied by the emitted macro function block.
fn macro_lines(macro_fns: &str) -> u32 {
    u32::try_from(macro_fns.lines().count()).unwrap_or(u32::MAX)
}

/// Names of the macros declared at the template top level.
fn macro_names(children: &[IrNode]) -> Vec<String> {
    children
        .iter()
        .filter_map(|n| match n {
            IrNode::StmtMacro { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect()
}

/// One `from <path> import <name> [as <local>]` binding resolved against the
/// catalog.
#[derive(Clone)]
struct MacroImport {
    /// Canonical id of the module that declares the macro.
    dep_id: String,
    /// Macro name as declared by the dependency.
    imported: String,
    /// Name the importing template calls it by.
    local: String,
}

/// Resolve every top-level `from ... import` to a canonical module + macro.
///
/// Non-literal paths, unresolvable references, unknown macro names, and
/// collisions with locally declared macros are compile diagnostics — the same
/// failures the interpreter raises while loading the catalog.
fn resolve_macro_imports(
    catalog: &TemplateCatalog,
    children: &[IrNode],
    from_id: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<MacroImport> {
    let mut out = Vec::new();
    let mut local_names: HashSet<String> = macro_names(children).into_iter().collect();
    for node in children {
        let IrNode::StmtImport { path, names, .. } = node else {
            continue;
        };
        let Some(lit) = crate::catalog::extract_path_literal(path) else {
            diagnostics.push(
                Diagnostic::new(Severity::Error, codes::IMPORT_PATH_NOT_LITERAL, Stage::Compile)
                    .template_id(from_id)
                    .message("`from ... import` requires a literal template path"),
            );
            continue;
        };
        let hit = match catalog.resolve_ref(&lit, Some(from_id)) {
            Ok(hit) => hit,
            Err(diag) => {
                diagnostics.push(diag.template_id(from_id));
                continue;
            }
        };
        let Some(dep_id) = catalog.canonical_id(&hit) else {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Error,
                    "dejavu.compile.template-missing",
                    Stage::Compile,
                )
                .template_id(from_id)
                .message(format!("template {lit:?} imported but not found in the catalog")),
            );
            continue;
        };
        let available = declared_macro_names(catalog, &dep_id);
        for spec in names {
            if !available.contains(&spec.name) {
                diagnostics.push(
                    Diagnostic::new(Severity::Error, codes::UNKNOWN_IMPORT, Stage::Compile)
                        .template_id(from_id)
                        .message(format!(
                            "template {dep_id:?} does not declare macro `{}`",
                            spec.name
                        ))
                        .arg("path", lit.clone())
                        .arg("name", spec.name.clone()),
                );
                continue;
            }
            let local = spec.local().to_string();
            if !local_names.insert(local.clone()) {
                diagnostics.push(
                    Diagnostic::new(Severity::Error, codes::MACRO_CONFLICT, Stage::Compile)
                        .template_id(from_id)
                        .message(format!("macro name `{local}` is already bound in this template"))
                        .arg("name", local.clone()),
                );
                continue;
            }
            out.push(MacroImport { dep_id: dep_id.clone(), imported: spec.name.clone(), local });
        }
    }
    out
}

/// Top-level macro names declared by a catalog template (own declarations only).
fn declared_macro_names(catalog: &TemplateCatalog, id: &str) -> HashSet<String> {
    match catalog_document(catalog, id) {
        Some(IrDocument { body: IrNode::Template { children, .. }, .. }) => {
            macro_names(children).into_iter().collect()
        }
        _ => HashSet::new(),
    }
}

/// ESM imports for `from ... import` bindings.
fn build_macro_import_lines(
    macro_imports: &[MacroImport],
    current_id: &str,
    id_to_path: &serde_json::Map<String, Value>,
) -> Vec<String> {
    let current_path = match id_to_path.get(current_id) {
        Some(Value::String(p)) => p.as_str(),
        _ => "",
    };
    let mut out = Vec::new();
    for imported in macro_imports {
        let Some(Value::String(target_path)) = id_to_path.get(&imported.dep_id) else {
            continue;
        };
        let rel = relative_import_path(current_path, target_path);
        out.push(format!(
            "import {{ {} as {} }} from \"{rel}\";",
            emit::macro_fn_name(&imported.imported),
            emit::macro_import_binding(&imported.local),
        ));
    }
    out
}

/// Exports that make this module's live macros importable by other templates.
fn macro_export_lines(live_macros: &[String], macro_imports: &[MacroImport]) -> Vec<String> {
    let mut out = Vec::new();
    for name in live_macros {
        out.push(format!("export {{ {} }};", emit::macro_fn_name(name)));
    }
    for imported in macro_imports {
        out.push(format!(
            "export {{ {} as {} }};",
            emit::macro_import_binding(&imported.local),
            emit::macro_fn_name(&imported.local),
        ));
    }
    out
}

/// Move body-line origins to module-line positions.
fn shift_origins(origins: &[LineOrigin], offset: u32) -> Vec<LineOrigin> {
    origins
        .iter()
        .map(|origin| LineOrigin {
            line: origin.line.saturating_add(offset),
            start: origin.start,
            end: origin.end,
        })
        .collect()
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
        // A partial that includes itself is recursion, not a module edge. The
        // module already binds its own `__tmpl_` alias, so no import is needed.
        if dep_id == current_id {
            continue;
        }
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
            // Included templates inside a partial or block body are still real
            // module dependencies.
            IrNode::StmtPartial { body, .. }
            | IrNode::StmtMacro { body, .. }
            | IrNode::StmtCall { body, .. }
            | IrNode::StmtBlock { body, .. } => {
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
