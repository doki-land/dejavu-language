//! path-v1 模板引用解析与 catalog loader —— Rust 侧唯一实现。
//!
//! 宿主只提供文件内容与 roots 配置；scheme 判定、相对路径拼接、
//! priority 排序、扩展名探测与 escape-root 拒绝都在这一个模块里完成。
//! 旧 TypeScript loader 的语义按位移植，诊断改为稳定 `code` + 结构化 `args`。

use std::collections::BTreeMap;

use crate::render::{ResolvedTemplate, TemplateLoader};
use crate::{Diagnostic, IrDocument, IrError, Severity, Stage, codes};

/// 单个 root 内的 canonical 路径 → 已解析 IR。
pub type CatalogFiles = BTreeMap<String, IrDocument>;

/// 解析用的 root 视图（只关心路径集合）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveRoot {
    /// root 名称，同时是默认 scheme。
    pub name: String,
    /// 可用于 `scheme:path` 的 scheme。
    pub scheme: String,
    /// 数字越大越优先被无 scheme 引用命中。
    pub priority: i32,
    /// 该 root 下的所有 canonical 路径。
    pub files: Vec<String>,
}

/// 模板 catalog root。
#[derive(Debug, Clone)]
pub struct CatalogRoot {
    /// root 名称，同时是默认 scheme。
    pub name: String,
    /// 显式 scheme。
    pub scheme: String,
    /// 数字越大越优先。
    pub priority: i32,
    /// canonical 路径 → IR 文档。
    pub files: CatalogFiles,
    /// 可选的宿主文件系统前缀（只作展示/追溯）。
    pub base_path: Option<String>,
}

impl CatalogRoot {
    /// 构造一个 root。
    pub fn new(name: impl Into<String>, scheme: impl Into<String>, priority: i32) -> Self {
        Self { name: name.into(), scheme: scheme.into(), priority, files: CatalogFiles::new(), base_path: None }
    }

    /// 登记一个已解析的模板。
    pub fn insert(&mut self, path: impl Into<String>, document: IrDocument) {
        self.files.insert(path.into(), document);
    }

    fn as_resolve_root(&self) -> ResolveRoot {
        ResolveRoot {
            name: self.name.clone(),
            scheme: self.scheme.clone(),
            priority: self.priority,
            files: self.files.keys().cloned().collect(),
        }
    }
}

/// 模板 catalog：一组 root + 扩展名探测表。
#[derive(Debug, Clone, Default)]
pub struct TemplateCatalog {
    /// 全部 root。
    pub roots: Vec<CatalogRoot>,
    /// 扩展名探测顺序（空字符串表示精确路径）。
    pub extensions: Vec<String>,
}

/// 命中结果：root 下标 + canonical 路径。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveHit {
    /// `TemplateCatalog::roots` 的下标。
    pub root_index: usize,
    /// root 内的 canonical 路径。
    pub path: String,
}

impl TemplateCatalog {
    /// 构造 catalog。
    pub fn new(roots: Vec<CatalogRoot>, extensions: Vec<String>) -> Self {
        Self { roots, extensions }
    }

    /// 解析一个模板引用。
    pub fn resolve_ref(&self, reference: &str, from: Option<&str>) -> Result<ResolveHit, Diagnostic> {
        let roots: Vec<ResolveRoot> = self.roots.iter().map(CatalogRoot::as_resolve_root).collect();
        resolve_ref(&roots, &self.extensions, reference, from)
    }

    /// 取命中路径对应的 IR。
    pub fn document(&self, hit: &ResolveHit) -> Option<&IrDocument> {
        self.roots.get(hit.root_index)?.files.get(&hit.path)
    }

    /// 取 root 名。
    pub fn root_name(&self, hit: &ResolveHit) -> Option<&str> {
        self.roots.get(hit.root_index).map(|r| r.name.as_str())
    }

    /// canonical id（`root:path`）。
    pub fn canonical_id(&self, hit: &ResolveHit) -> Option<String> {
        self.root_name(hit).map(|name| canonical_id(name, &hit.path))
    }
}

/// 拼 canonical id。
pub fn canonical_id(root_name: &str, path: &str) -> String {
    format!("{root_name}:{path}")
}

/// 解析引用：catalog 索引 + ref + 可选 canonical `from`。
pub fn resolve_ref(
    roots: &[ResolveRoot],
    extensions: &[String],
    reference: &str,
    from: Option<&str>,
) -> Result<ResolveHit, Diagnostic> {
    let raw = reference.replace('\\', "/");
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(load_diag(codes::TEMPLATE_INVALID_REF, reference).arg("reason", "empty-ref"));
    }

    if let Some((scheme, path)) = split_scheme(raw) {
        let Some(index) = root_by_scheme(roots, scheme) else {
            return Err(load_diag(codes::TEMPLATE_UNKNOWN_SCHEME, reference).arg("scheme", scheme));
        };
        return match probe_root(&roots[index], path, extensions) {
            Some(path) => Ok(ResolveHit { root_index: index, path }),
            None => Err(not_found(reference, None, vec![roots[index].name.clone()])),
        };
    }

    if raw.starts_with("./") || raw.starts_with("../") {
        let Some(from) = from else {
            return Err(load_diag(codes::TEMPLATE_RELATIVE_WITHOUT_FROM, reference));
        };
        let Some((from_index, from_path)) = parse_canonical_id(roots, from) else {
            return Err(load_diag(codes::TEMPLATE_INVALID_REF, reference)
                .arg("reason", "non-canonical-from")
                .arg("from", from));
        };
        let Ok(joined) = join_posix(&dirname_posix(&from_path), raw) else {
            return Err(load_diag(codes::TEMPLATE_RELATIVE_ESCAPE_ROOT, reference).arg("from", from));
        };
        return match probe_root(&roots[from_index], &joined, extensions) {
            Some(path) => Ok(ResolveHit { root_index: from_index, path }),
            None => Err(not_found(reference, Some(from), vec![roots[from_index].name.clone()])),
        };
    }

    let mut ranked: Vec<usize> = (0..roots.len()).collect();
    ranked.sort_by(|a, b| roots[*b].priority.cmp(&roots[*a].priority));

    // Walk the roots priority-descending. Hits inside one priority level are
    // peers: more than one hit there is an ambiguity, never a silent order win.
    let mut searched: Vec<String> = Vec::new();
    let mut index = 0;
    while index < ranked.len() {
        let priority = roots[ranked[index]].priority;
        let mut hits: Vec<(usize, String)> = Vec::new();
        while index < ranked.len() && roots[ranked[index]].priority == priority {
            let root_index = ranked[index];
            searched.push(roots[root_index].name.clone());
            if let Some(path) = probe_root(&roots[root_index], raw, extensions) {
                hits.push((root_index, path));
            }
            index += 1;
        }
        match hits.len() {
            0 => continue,
            1 => {
                let (root_index, path) = hits.remove(0);
                return Ok(ResolveHit { root_index, path });
            }
            _ => {
                return Err(ambiguous_ref(reference, from, priority, &hits, roots));
            }
        }
    }
    Err(not_found(reference, from, searched))
}

/// catalog-backed [`TemplateLoader`]。
pub struct CatalogLoader<'a> {
    /// 被借用的 catalog。
    pub catalog: &'a TemplateCatalog,
}

impl CatalogLoader<'_> {
    /// 用 catalog 构造 loader。
    pub fn new(catalog: &TemplateCatalog) -> CatalogLoader<'_> {
        CatalogLoader { catalog }
    }
}

impl TemplateLoader for CatalogLoader<'_> {
    fn resolve(&mut self, ref_: &str, from: Option<&str>) -> Result<ResolvedTemplate, IrError> {
        let hit = self.catalog.resolve_ref(ref_, from)?;
        let Some(document) = self.catalog.document(&hit) else {
            return Err(IrError::Diagnostic(
                load_diag(codes::TEMPLATE_NOT_FOUND, ref_).arg("reason", "document-missing").arg("path", &hit.path),
            ));
        };
        let Some(id) = self.catalog.canonical_id(&hit) else {
            return Err(IrError::Diagnostic(load_diag(codes::TEMPLATE_NOT_FOUND, ref_)));
        };
        Ok(ResolvedTemplate { id, document: document.clone() })
    }
}

// ---------------------------------------------------------------------------
// 内部辅助
// ---------------------------------------------------------------------------

fn load_diag(code: &str, reference: &str) -> Diagnostic {
    Diagnostic::new(Severity::Error, code, Stage::Load).arg("ref", reference)
}

fn not_found(reference: &str, from: Option<&str>, searched: Vec<String>) -> Diagnostic {
    let mut diag = load_diag(codes::TEMPLATE_NOT_FOUND, reference)
        .arg("searched", searched.into_iter().map(crate::ArgValue::String).collect::<Vec<_>>());
    if let Some(from) = from {
        diag = diag.arg("from", from);
    }
    diag
}

/// 同一 priority 下多个 root 命中同一 bare 引用：拒绝按 catalog 顺序静默取胜。
fn ambiguous_ref(
    reference: &str,
    from: Option<&str>,
    priority: i32,
    hits: &[(usize, String)],
    roots: &[ResolveRoot],
) -> Diagnostic {
    let names: Vec<crate::ArgValue> = hits
        .iter()
        .map(|(index, _)| crate::ArgValue::String(roots[*index].name.clone()))
        .collect();
    let candidates: Vec<String> = hits
        .iter()
        .map(|(index, path)| canonical_id(&roots[*index].name, path))
        .collect();
    let mut diag = load_diag(codes::TEMPLATE_AMBIGUOUS_REF, reference)
        .message(format!(
            "template reference {reference:?} matches {} roots at priority {priority}: {}. \
             Use an explicit `root:path` reference.",
            candidates.len(),
            candidates.join(", "),
        ))
        .arg("priority", priority as i64)
        .arg("roots", names)
        .arg("candidates", candidates.into_iter().map(crate::ArgValue::String).collect::<Vec<_>>());
    if let Some(from) = from {
        diag = diag.arg("from", from);
    }
    diag
}

/// `^([A-Za-z][A-Za-z0-9_+-]*):(.*)$`
fn split_scheme(raw: &str) -> Option<(&str, &str)> {
    let index = raw.find(':')?;
    let (scheme, rest) = raw.split_at(index);
    let mut chars = scheme.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '-') {
        return None;
    }
    Some((scheme, &rest[1..]))
}

fn root_by_scheme(roots: &[ResolveRoot], scheme: &str) -> Option<usize> {
    roots.iter().rposition(|r| r.scheme == scheme)
}

fn root_by_name(roots: &[ResolveRoot], name: &str) -> Option<usize> {
    roots.iter().rposition(|r| r.name == name)
}

fn parse_canonical_id(roots: &[ResolveRoot], id: &str) -> Option<(usize, String)> {
    let (name, path) = split_scheme(id)?;
    let index = root_by_name(roots, name).or_else(|| root_by_scheme(roots, name))?;
    let normalized = posix_normalize(path).ok()?;
    Some((index, normalized))
}

fn probe_root(root: &ResolveRoot, path: &str, extensions: &[String]) -> Option<String> {
    let normalized = posix_normalize(path).ok()?;
    let has_known_ext = extensions.iter().any(|e| !e.is_empty() && normalized.ends_with(e.as_str()));
    if has_known_ext {
        return root.files.iter().any(|key| key == &normalized).then_some(normalized);
    }
    for ext in extensions {
        let key = format!("{normalized}{ext}");
        if root.files.iter().any(|known| known == &key) {
            return Some(key);
        }
    }
    None
}

/// POSIX 归一化；越出 root 时返回 `Err`。
pub fn posix_normalize(path: &str) -> Result<String, ()> {
    let mut parts: Vec<&str> = Vec::new();
    let unified = path.replace('\\', "/");
    for segment in unified.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            if parts.pop().is_none() {
                return Err(());
            }
            continue;
        }
        parts.push(segment);
    }
    Ok(parts.join("/"))
}

fn dirname_posix(path: &str) -> String {
    match path.rfind('/') {
        Some(index) => path[..index].to_string(),
        None => String::new(),
    }
}

fn join_posix(dir: &str, rel: &str) -> Result<String, ()> {
    if dir.is_empty() {
        return posix_normalize(rel);
    }
    posix_normalize(&format!("{dir}/{rel}"))
}
