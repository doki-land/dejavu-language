//! Catalog-aware helpers for multi-template AOT codegen.
//!
//! Things that only make sense inside `precompile_catalog`:
//! - Parsing the catalog JSON wire format → [`TemplateCatalog`]
//! - Template id → JS identifier + file path sanitization
//! - Include / extends cycle detection (DFS on canonical ids)
//! - Block collection along an extends chain
//! - Path-literal extraction from `Stmt.Include` / `Stmt.Extends`

use std::collections::{HashMap, HashSet};

use dejavu_diagnostic::{Diagnostic, Severity, Stage};
use dejavu_ir::{CatalogRoot, IrDocument, IrNode, TemplateCatalog};

// ---------------------------------------------------------------------------
// Catalog JSON wire format
// ---------------------------------------------------------------------------

/// JSON-deserializable shadow of `dejavu_ir::TemplateCatalog`.
///
/// We keep our own shadow because we want the field names in the JSON wire
/// (`roots`, `files`) to match the TS side *exactly*, without accidentally
/// depending on what the Rust crate exports.
#[derive(Debug, serde::Deserialize)]
struct CatalogJson {
    roots: Vec<RootJson>,
    #[serde(default)]
    extensions: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct RootJson {
    name: String,
    #[serde(default)]
    scheme: Option<String>,
    #[serde(default)]
    priority: Option<i32>,
    /// Path → IR JSON *string* (wire format). Values are strings because
    /// serde_json won't let us build IrDocument directly inside a `BTreeMap`
    /// while parsing — we stringify and reparse per entry below.
    files: HashMap<String, String>,
    #[serde(default)]
    base_path: Option<String>,
}

/// Parse a catalog JSON string into a fully-resolved [`TemplateCatalog`].
pub fn parse_catalog_json(json: &str) -> Result<TemplateCatalog, String> {
    let cj: CatalogJson = serde_json::from_str(json)
        .map_err(|e| format!("invalid catalog JSON: {e}"))?;

    let extensions = if cj.extensions.is_empty() {
        vec![".dejavu".to_string(), ".yarte".to_string(), String::new()]
    } else {
        cj.extensions
    };

    let mut roots = Vec::with_capacity(cj.roots.len());
    for r in cj.roots {
        let scheme = r.scheme.clone().unwrap_or_else(|| r.name.clone());
        let priority = r.priority.unwrap_or(10);
        let mut root = CatalogRoot::new(r.name.clone(), scheme, priority);
        root.base_path = r.base_path;
        for (path, ir_json) in r.files {
            let doc: IrDocument = serde_json::from_str(&ir_json)
                .map_err(|e| format!("invalid IR JSON for {}: {}", path, e))?;
            root.insert(path, doc);
        }
        roots.push(root);
    }

    Ok(TemplateCatalog::new(roots, extensions))
}

// ---------------------------------------------------------------------------
// Sanitize: canonical id → JS identifier + file base
// ---------------------------------------------------------------------------

/// JavaScript reserved words that we suffix with `_` during sanitization.
/// (Reuses the set in emit.rs — exposed here for the sanitize path.)
fn is_js_reserved(name: &str) -> bool {
    use once_cell::sync::Lazy;
    use std::collections::HashSet;
    static RESERVED: Lazy<HashSet<&'static str>> = Lazy::new(|| {
        let mut s = HashSet::new();
        for w in [
            "break", "case", "catch", "class", "const", "continue", "debugger",
            "default", "delete", "do", "else", "enum", "export", "extends",
            "false", "finally", "for", "function", "if", "import", "in",
            "instanceof", "new", "null", "return", "super", "switch", "this",
            "throw", "true", "try", "typeof", "var", "void", "while", "with",
            "yield", "await", "implements", "interface", "package", "private",
            "protected", "public",
        ] {
            s.insert(w);
        }
        s
    });
    RESERVED.contains(name)
}

/// Sanitize a canonical template id into a JS-friendly identifier fragment.
///
/// Rules:
/// - All non-identifier chars (`:`, `/`, `.`, `-`, spaces, etc.) → `_`
/// - If starts with a digit → prepend `_`
/// - If collides with a JS reserved word → append `_`
///
/// Example: `"main:index"` → `"main__index"`, `"partial:footer"` → `"partial__footer"`
pub fn sanitize_id(id: &str) -> String {
    let mut s: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if let Some(first) = s.chars().next() {
        if first.is_ascii_digit() {
            s.insert(0, '_');
        }
    }
    if is_js_reserved(&s) {
        s.push('_');
    }
    s
}

/// JS `import` binding name for a template id (prefixes `__tmpl_` to avoid
/// accidental collision with user-space identifiers).
pub fn js_import_name(id: &str) -> String {
    format!("__tmpl_{}", sanitize_id(id))
}

// ---------------------------------------------------------------------------
// Include/extends path literal extraction
// ---------------------------------------------------------------------------

/// If the `Stmt.Include.path` / `Stmt.Extends.parent` is an Expr.Literal
/// string node, return its string value. Otherwise `None` (dynamic — unsupported
/// in AOT catalog mode; callers should produce a diagnostic).
pub fn extract_path_literal(node: &IrNode) -> Option<String> {
    match node {
        IrNode::ExprLiteral { value: serde_json::Value::String(s), .. } => Some(s.clone()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Cycle detection
// ---------------------------------------------------------------------------

/// DFS through include + extends edges starting from `entry_id`.
///
/// Returns the canonical id of the first node that was already on the
/// current DFS stack, so callers can build a diagnostic message with the
/// cycle participant. `Ok(())` when no cycle is present.
pub fn detect_cycle(
    catalog: &TemplateCatalog,
    entry_id: &str,
) -> Result<(), Diagnostic> {
    let mut state: HashMap<String, CycleState> = HashMap::new();
    detect_cycle_inner(catalog, entry_id, &mut state)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CycleState {
    Visiting,
    Done,
}

fn detect_cycle_inner(
    catalog: &TemplateCatalog,
    id: &str,
    state: &mut HashMap<String, CycleState>,
) -> Result<(), Diagnostic> {
    let st = state.get(id).copied();
    match st {
        Some(CycleState::Visiting) => {
            return Err(
                Diagnostic::new(
                    Severity::Error,
                    "dejavu.compile.circular-reference",
                    Stage::Compile,
                )
                .template_id(id)
                .message(format!("circular include/extends involving {id}")),
            );
        }
        Some(CycleState::Done) => return Ok(()),
        None => {}
    }
    state.insert(id.to_string(), CycleState::Visiting);

    let edges = collect_edges(catalog, id, false);
    for next in edges {
        detect_cycle_inner(catalog, &next, state)?;
    }

    state.insert(id.to_string(), CycleState::Done);
    Ok(())
}

/// Gather all canonical ids reachable from `id` via a single include, extends,
/// or (when `follow_imports`) `from ... import` edge. Missing templates are
/// skipped — we want to surface cycle errors only when both endpoints exist.
///
/// `from ... import` edges are followed for reachability but excluded from
/// cycle detection: ESM function declarations are hoisted, so two templates
/// may import each other's macros the way the interpreter allows.
fn collect_edges(catalog: &TemplateCatalog, id: &str, follow_imports: bool) -> Vec<String> {
    let mut out = Vec::new();
    let Some((root_name, path)) = split_canonical(id) else { return out };
    // Find the right root + path in catalog.
    let doc = catalog_document(catalog, root_name, path);
    let Some(doc) = doc else { return out };
    collect_edges_from_doc(catalog, doc, id, follow_imports, &mut out);
    out
}

fn collect_edges_from_doc(
    catalog: &TemplateCatalog,
    doc: &IrDocument,
    from_id: &str,
    follow_imports: bool,
    out: &mut Vec<String>,
) {
    let body = match &doc.body {
        IrNode::Template { children, .. } => children,
        _ => return,
    };
    for n in body {
        collect_edges_nodes(catalog, n, from_id, follow_imports, out);
    }
}

/// Walk nested bodies too: an `include` inside a partial or block body is a
/// real module dependency and must enter the reachable set.
fn collect_edges_nodes(
    catalog: &TemplateCatalog,
    node: &IrNode,
    from_id: &str,
    follow_imports: bool,
    out: &mut Vec<String>,
) {
    match node {
        IrNode::StmtInclude { path, .. } => {
            push_ref_edge(catalog, path, from_id, out);
        }
        IrNode::StmtImport { path, .. } => {
            if follow_imports {
                push_ref_edge(catalog, path, from_id, out);
            }
        }
        IrNode::StmtExtends { parent, .. } => {
            push_ref_edge(catalog, parent, from_id, out);
        }
        IrNode::Template { children, .. }
        | IrNode::StmtFor { body: children, .. }
        | IrNode::StmtPartial { body: children, .. }
        | IrNode::StmtMacro { body: children, .. }
        | IrNode::StmtBlock { body: children, .. } => {
            for child in children {
                collect_edges_nodes(catalog, child, from_id, follow_imports, out);
            }
        }
        IrNode::StmtIf { test, consequent, alternate, .. } => {
            collect_edges_nodes(catalog, test, from_id, follow_imports, out);
            for child in consequent {
                collect_edges_nodes(catalog, child, from_id, follow_imports, out);
            }
            if let Some(alt) = alternate {
                for child in alt {
                    collect_edges_nodes(catalog, child, from_id, follow_imports, out);
                }
            }
        }
        _ => {}
    }
}

/// Resolve an include / extends / import path literal to a canonical id.
fn push_ref_edge(
    catalog: &TemplateCatalog,
    path: &IrNode,
    from_id: &str,
    out: &mut Vec<String>,
) {
    if let Some(p) = extract_path_literal(path) {
        if let Ok(hit) = catalog.resolve_ref(&p, Some(from_id)) {
            if let Some(canon) = catalog.canonical_id(&hit) {
                out.push(canon);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Block collection (along extends chain)
// ---------------------------------------------------------------------------

/// Follow the extends chain starting at `entry_id` and return the canonical id
/// of the **topmost** parent — that's where the render body should start from
/// when flattening the entry template. If the entry has no extends it returns
/// the entry itself.
pub fn collect_blocks(
    catalog: &TemplateCatalog,
    entry_id: &str,
) -> Result<String, Diagnostic> {
    let mut cur = entry_id.to_string();
    let mut topmost = entry_id.to_string();
    loop {
        let Some(parent_ref) = doc_extends_ref(catalog, &cur) else { break };
        let hit = catalog
            .resolve_ref(&parent_ref, Some(&cur))
            .map_err(|d| d.template_id(&cur))?;
        let Some(parent_id) = catalog.canonical_id(&hit) else { break };
        topmost = parent_id.clone();
        cur = parent_id;
    }
    Ok(topmost)
}

/// Collect every ancestor definition of each block, most-derived first.
///
/// `super` walks one entry further down the chain, so a three-level template
/// chain (`child → mid → base`) can resolve `super` in the child to `mid`
/// and `super` in `mid` to `base`.
pub fn collect_block_chains(
    catalog: &TemplateCatalog,
    entry_id: &str,
) -> Result<HashMap<String, Vec<Vec<IrNode>>>, Diagnostic> {
    let mut chain: Vec<String> = vec![entry_id.to_string()];
    let mut cur = entry_id.to_string();
    loop {
        let Some(r) = doc_extends_ref(catalog, &cur) else { break };
        let hit = catalog.resolve_ref(&r, Some(&cur))
            .map_err(|d| d.template_id(&cur))?;
        let Some(id) = catalog.canonical_id(&hit) else { break };
        chain.push(id.clone());
        cur = id;
    }
    let mut chains: HashMap<String, Vec<Vec<IrNode>>> = HashMap::new();
    for id in &chain {
        let Some((root_name, path)) = split_canonical(id) else { continue };
        let Some(doc) = catalog_document(catalog, root_name, path) else { continue };
        if let IrNode::Template { children, .. } = &doc.body {
            for c in children {
                if let IrNode::StmtBlock { name, body, .. } = c {
                    chains.entry(name.clone()).or_default().push(body.clone());
                }
            }
        }
    }
    Ok(chains)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the `Stmt.Extends.parent` literal for a template, if any.
fn doc_extends_ref(catalog: &TemplateCatalog, id: &str) -> Option<String> {
    let (root_name, path) = split_canonical(id)?;
    let doc = catalog_document(catalog, root_name, path)?;
    if let IrNode::Template { children, .. } = &doc.body {
        for c in children {
            if let IrNode::StmtExtends { parent, .. } = c {
                return extract_path_literal(parent);
            }
        }
    }
    None
}

pub fn split_canonical(id: &str) -> Option<(&str, &str)> {
    let idx = id.find(':')?;
    let (root, rest) = id.split_at(idx);
    if root.is_empty() || rest.is_empty() {
        return None;
    }
    Some((root, &rest[1..]))
}

fn catalog_document<'a>(
    catalog: &'a TemplateCatalog,
    root_name: &str,
    path: &str,
) -> Option<&'a IrDocument> {
    for r in &catalog.roots {
        if r.name == root_name {
            return r.files.get(path);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Collect all template ids reachable from an entry
// ---------------------------------------------------------------------------

/// Walk include + extends edges and return the set of all canonical ids
/// transitively reachable from `entry_id` (including `entry_id` itself).
pub fn collect_all_reachable(
    catalog: &TemplateCatalog,
    entry_id: &str,
) -> HashSet<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut stack: Vec<String> = vec![entry_id.to_string()];
    while let Some(id) = stack.pop() {
        if !seen.insert(id.clone()) { continue; }
        for next in collect_edges(catalog, &id, true) {
            if !seen.contains(&next) {
                stack.push(next);
            }
        }
    }
    seen
}
