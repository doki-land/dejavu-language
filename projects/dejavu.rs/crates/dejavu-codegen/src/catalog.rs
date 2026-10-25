//! Catalog-aware helpers for multi-template AOT codegen.
//!
//! Things that only make sense inside `precompile_catalog`:
//! - Parsing the catalog JSON wire format → [`TemplateCatalog`]
//! - Template id → JS identifier + file path sanitization
//! - Include / extends cycle detection (DFS on canonical ids)
//! - Block collection along an extends chain
//! - Path-literal extraction from `Stmt.Include` / `Stmt.Extends`

use std::collections::{HashMap, HashSet};

use dejavu_diagnostic::{codes, Diagnostic, Severity, Stage};
use dejavu_ir::{CatalogRoot, IrDocument, IrNode, TemplateCatalog};
use dejavu_ir::types::PartialParam;

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
/// The closing edge decides the diagnostic: an `extends` edge reports
/// `dejavu.template.circular-extends` with the parent id in `chain`, an
/// `include` edge reports `dejavu.template.circular-include` with the
/// `a -> b -> a` path. Both match the interpreter's load-stage diagnostics so
/// AOT and JIT agree on `code` and `args`. `Ok(())` when no cycle is present.
pub fn detect_cycle(
    catalog: &TemplateCatalog,
    entry_id: &str,
) -> Result<(), Diagnostic> {
    let mut state: HashMap<String, CycleState> = HashMap::new();
    let mut path: Vec<String> = Vec::new();
    detect_cycle_inner(catalog, entry_id, &mut state, &mut path)
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
    path: &mut Vec<String>,
) -> Result<(), Diagnostic> {
    match state.get(id).copied() {
        Some(CycleState::Done) => return Ok(()),
        Some(CycleState::Visiting) => {
            // Back edges are classified where they are traversed, not here.
            return Ok(());
        }
        None => {}
    }
    state.insert(id.to_string(), CycleState::Visiting);
    path.push(id.to_string());

    for (next, kind) in collect_edges(catalog, id, false) {
        if state.get(&next).copied() == Some(CycleState::Visiting) {
            if let Some(kind) = kind {
                return Err(circular_diagnostic(kind, &next, path));
            }
            continue;
        }
        detect_cycle_inner(catalog, &next, state, path)?;
    }

    path.pop();
    state.insert(id.to_string(), CycleState::Done);
    Ok(())
}

/// Build the load-stage cycle diagnostic for the edge that closes the cycle.
///
/// The `chain` arg reproduces what the interpreter reports at the moment it
/// detects the same cycle: the include stack never holds the entry template,
/// so it is re-entered before the repeat shows up, while an extends cycle
/// reports the parent that was already being extended.
fn circular_diagnostic(kind: EdgeKind, target: &str, path: &[String]) -> Diagnostic {
    let start = path.iter().position(|id| id == target).unwrap_or(0);
    match kind {
        EdgeKind::Extends => {
            let repeated = path.get(start + 1).cloned().unwrap_or_else(|| target.to_string());
            Diagnostic::new(Severity::Error, codes::CIRCULAR_EXTENDS, Stage::Load)
                .template_id(target)
                .arg("chain", repeated)
                .message(format!("circular extends involving {target}"))
        }
        EdgeKind::Include => {
            let mut chain: Vec<String> = path[1.min(path.len())..].to_vec();
            chain.push(target.to_string());
            if start == 0 {
                chain.push(path.get(1).cloned().unwrap_or_else(|| target.to_string()));
            }
            Diagnostic::new(Severity::Error, codes::CIRCULAR_INCLUDE, Stage::Load)
                .template_id(target)
                .arg("chain", chain.join(" -> "))
                .message(format!("circular include involving {target}"))
        }
    }
}

/// Which statement produced an edge. `None` marks a `from ... import` edge,
/// which is reachability-only and never closes a cycle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EdgeKind {
    Include,
    Extends,
}

/// Gather all canonical ids reachable from `id` via a single include, extends,
/// or (when `follow_imports`) `from ... import` edge. Missing templates are
/// skipped — we want to surface cycle errors only when both endpoints exist.
///
/// `from ... import` edges are followed for reachability but excluded from
/// cycle detection: ESM function declarations are hoisted, so two templates
/// may import each other's macros the way the interpreter allows.
fn collect_edges(
    catalog: &TemplateCatalog,
    id: &str,
    follow_imports: bool,
) -> Vec<(String, Option<EdgeKind>)> {
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
    out: &mut Vec<(String, Option<EdgeKind>)>,
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
    out: &mut Vec<(String, Option<EdgeKind>)>,
) {
    match node {
        IrNode::StmtInclude { path, .. } => {
            push_ref_edge(catalog, path, from_id, Some(EdgeKind::Include), out);
        }
        IrNode::StmtImport { path, .. } => {
            if follow_imports {
                push_ref_edge(catalog, path, from_id, None, out);
            }
        }
        IrNode::StmtExtends { parent, .. } => {
            push_ref_edge(catalog, parent, from_id, Some(EdgeKind::Extends), out);
        }
        IrNode::Template { children, .. }
        | IrNode::StmtFor { body: children, .. }
        | IrNode::StmtPartial { body: children, .. }
        | IrNode::StmtMacro { body: children, .. }
        | IrNode::StmtCall { body: children, .. }
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
    kind: Option<EdgeKind>,
    out: &mut Vec<(String, Option<EdgeKind>)>,
) {
    if let Some(p) = extract_path_literal(path) {
        if let Ok(hit) = catalog.resolve_ref(&p, Some(from_id)) {
            if let Some(canon) = catalog.canonical_id(&hit) {
                out.push((canon, kind));
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
        let parent_ref = match doc_extends_ref(catalog, &cur) {
            ExtendsRef::None => break,
            ExtendsRef::Dynamic => return Err(extends_path_not_literal(&cur)),
            ExtendsRef::Literal(lit) => lit,
        };
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
        let r = match doc_extends_ref(catalog, &cur) {
            ExtendsRef::None => break,
            ExtendsRef::Dynamic => return Err(extends_path_not_literal(&cur)),
            ExtendsRef::Literal(lit) => lit,
        };
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

/// The top-level `Stmt.Extends` of a template, classified by its target.
enum ExtendsRef {
    None,
    /// Literal template path — resolvable inside the catalog.
    Literal(String),
    /// Dynamic target — the renderer and AOT both reject it.
    Dynamic,
}

/// Classify the `Stmt.Extends.parent` of a template.
fn doc_extends_ref(catalog: &TemplateCatalog, id: &str) -> ExtendsRef {
    let Some((root_name, path)) = split_canonical(id) else { return ExtendsRef::None };
    let Some(doc) = catalog_document(catalog, root_name, path) else { return ExtendsRef::None };
    if let IrNode::Template { children, .. } = &doc.body {
        for c in children {
            if let IrNode::StmtExtends { parent, .. } = c {
                return match extract_path_literal(parent) {
                    Some(lit) => ExtendsRef::Literal(lit),
                    None => ExtendsRef::Dynamic,
                };
            }
        }
    }
    ExtendsRef::None
}

/// `extends` targets must be literal paths; the renderer raises the same code
/// at load time.
fn extends_path_not_literal(id: &str) -> Diagnostic {
    Diagnostic::new(Severity::Error, codes::EXTENDS_PATH_NOT_LITERAL, Stage::Compile)
        .template_id(id)
        .message("`extends` requires a literal template path")
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
        for (next, _) in collect_edges(catalog, &id, true) {
            if !seen.contains(&next) {
                stack.push(next);
            }
        }
    }
    seen
}

// ---------------------------------------------------------------------------
// Macro call candidates (tree-shaking input)
// ---------------------------------------------------------------------------

/// Collect every identifier that appears as a macro-call candidate.
///
/// Candidates are bare-identifier `Expr.Call` callees and `Stmt.Call` names;
/// whether a candidate really is a macro depends on the module's macro table,
/// which is resolved by [`crate::codegen`]. The walker only records names, so
/// it is deliberately independent of macro declarations.
///
/// `skip_macro_bodies` excludes `Stmt.Macro` bodies: a declaration produces no
/// output in the enclosing render body, so calls inside it only matter once the
/// macro itself is reachable.
pub(crate) fn collect_macro_calls(
    nodes: &[IrNode],
    skip_macro_bodies: bool,
    out: &mut HashSet<String>,
) {
    for node in nodes {
        collect_macro_calls_node(node, skip_macro_bodies, out);
    }
}

fn collect_macro_calls_node(node: &IrNode, skip_macro_bodies: bool, out: &mut HashSet<String>) {
    match node {
        IrNode::ExprCall { callee, arguments, .. } => {
            if let IrNode::ExprIdentifier { name, .. } = callee.as_ref() {
                out.insert(name.clone());
            } else {
                collect_macro_calls_node(callee, skip_macro_bodies, out);
            }
            collect_macro_calls(arguments, skip_macro_bodies, out);
        }
        IrNode::StmtCall { name, arguments, body, .. } => {
            out.insert(name.clone());
            collect_macro_calls(arguments, skip_macro_bodies, out);
            collect_macro_calls(body, skip_macro_bodies, out);
        }
        IrNode::StmtMacro { params, body, .. } => {
            if !skip_macro_bodies {
                collect_param_defaults(params, out);
                collect_macro_calls(body, skip_macro_bodies, out);
            }
        }
        IrNode::StmtPartial { params, body, .. } => {
            collect_param_defaults(params, out);
            collect_macro_calls(body, skip_macro_bodies, out);
        }
        IrNode::Template { children, .. } => {
            collect_macro_calls(children, skip_macro_bodies, out);
        }
        IrNode::Interpolation { expression, .. } => {
            collect_macro_calls_node(expression, skip_macro_bodies, out);
        }
        IrNode::StmtIf { test, consequent, else_ifs, alternate, .. } => {
            collect_macro_calls_node(test, skip_macro_bodies, out);
            collect_macro_calls(consequent, skip_macro_bodies, out);
            for ei in else_ifs {
                collect_macro_calls_node(ei, skip_macro_bodies, out);
            }
            if let Some(alt) = alternate {
                collect_macro_calls(alt, skip_macro_bodies, out);
            }
        }
        IrNode::StmtElseIf { test, consequent, .. } => {
            collect_macro_calls_node(test, skip_macro_bodies, out);
            collect_macro_calls(consequent, skip_macro_bodies, out);
        }
        IrNode::StmtFor { iterable, body, .. } => {
            collect_macro_calls_node(iterable, skip_macro_bodies, out);
            collect_macro_calls(body, skip_macro_bodies, out);
        }
        IrNode::StmtBlock { body, .. } => {
            collect_macro_calls(body, skip_macro_bodies, out);
        }
        IrNode::StmtExtends { parent, .. } => {
            collect_macro_calls_node(parent, skip_macro_bodies, out);
        }
        IrNode::StmtInclude { path, with, .. } => {
            collect_macro_calls_node(path, skip_macro_bodies, out);
            if let Some(with) = with {
                collect_macro_calls_node(with, skip_macro_bodies, out);
            }
        }
        IrNode::ExprMember { object, .. } => {
            collect_macro_calls_node(object, skip_macro_bodies, out);
        }
        IrNode::ExprIndex { object, index, .. } => {
            collect_macro_calls_node(object, skip_macro_bodies, out);
            collect_macro_calls_node(index, skip_macro_bodies, out);
        }
        IrNode::ExprTest { expression, arguments, .. } => {
            collect_macro_calls_node(expression, skip_macro_bodies, out);
            collect_macro_calls(arguments, skip_macro_bodies, out);
        }
        IrNode::ExprBinary { left, right, .. } => {
            collect_macro_calls_node(left, skip_macro_bodies, out);
            collect_macro_calls_node(right, skip_macro_bodies, out);
        }
        IrNode::ExprUnary { argument, .. } => {
            collect_macro_calls_node(argument, skip_macro_bodies, out);
        }
        IrNode::ExprObject { entries, .. } => {
            for entry in entries {
                collect_macro_calls_node(&entry.value, skip_macro_bodies, out);
            }
        }
        IrNode::ExprPipe { expression, arguments, .. } => {
            collect_macro_calls_node(expression, skip_macro_bodies, out);
            collect_macro_calls(arguments, skip_macro_bodies, out);
        }
        // Declarations and leaves cannot contain a call site (the `import` /
        // `extends` / `include` paths are literals by contract).
        IrNode::StmtImport { .. }
        | IrNode::StmtSuper { .. }
        | IrNode::StmtRaw { .. }
        | IrNode::Text { .. }
        | IrNode::Comment { .. }
        | IrNode::ExprLiteral { .. }
        | IrNode::ExprIdentifier { .. } => {}
    }
}

fn collect_param_defaults(params: &[PartialParam], out: &mut HashSet<String>) {
    for param in params {
        if let Some(default) = &param.default {
            // Defaults are literals by contract; walk them anyway so a future
            // expression default cannot silently escape tree-shaking.
            collect_macro_calls_node(default, false, out);
        }
    }
}
