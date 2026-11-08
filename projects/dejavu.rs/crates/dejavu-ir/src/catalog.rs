//! path-v1 模板引用解析与 catalog loader —— dejavu 的宿主边界。
//!
//! 引用分类、相对路径拼接、扩展名探测与根内命中判定由共享
//! [`document_template::MapTemplateLoader`] 完成；本模块只保留多根优先级、
//! 同优先级歧义拒绝、scheme 别名翻译与 dejavu 冻结诊断这些宿主策略。

use std::collections::BTreeMap;

use document_template::{TemplateId, codes as shared};

use crate::{
    Diagnostic, IrDocument, IrError, Severity, Stage, codes,
    render::{ResolvedTemplate, TemplateLoader},
    template_bridge::RootIndex,
};

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

    let index = RootIndex::new(roots);
    let ext: Vec<&str> = extensions.iter().map(String::as_str).collect();

    if let Some((scheme, path)) = split_scheme(raw) {
        let Some(root_index) = root_by_scheme(roots, scheme)
        else {
            return Err(load_diag(codes::TEMPLATE_UNKNOWN_SCHEME, reference).arg("scheme", scheme));
        };
        // 共享合同只按 root `name` 认 scheme，别名在这里翻译成 `name:path`。
        return match index.resolve(root_index, path, None, &ext) {
            Ok(id) => Ok(ResolveHit { root_index, path: id.path().to_string() }),
            Err(error) => Err(shared_or_missing(error, reference, None, vec![roots[root_index].name.clone()])),
        };
    }

    if raw.starts_with("./") || raw.starts_with("../") {
        let Some(from) = from
        else {
            return Err(load_diag(codes::TEMPLATE_RELATIVE_WITHOUT_FROM, reference));
        };
        let Some((from_index, from_path)) = parse_canonical_id(roots, from)
        else {
            return Err(load_diag(codes::TEMPLATE_INVALID_REF, reference)
                .arg("reason", "non-canonical-from")
                .arg("from", from));
        };
        let from_id = TemplateId::new(roots[from_index].name.clone(), from_path);
        return match index.resolve(from_index, raw, Some(&from_id), &ext) {
            Ok(id) => Ok(ResolveHit { root_index: from_index, path: id.path().to_string() }),
            Err(error) => Err(shared_or_missing(error, reference, Some(from), vec![roots[from_index].name.clone()])),
        };
    }

    let mut ranked: Vec<usize> = (0..roots.len()).collect();
    ranked.sort_by(|a, b| roots[*b].priority.cmp(&roots[*a].priority));

    // Walk the roots priority-descending. Hits inside one priority level are
    // peers: more than one hit there is an ambiguity, never a silent order win.
    let mut searched: Vec<String> = Vec::new();
    let mut index_at = 0;
    while index_at < ranked.len() {
        let priority = roots[ranked[index_at]].priority;
        let mut hits: Vec<(usize, String)> = Vec::new();
        while index_at < ranked.len() && roots[ranked[index_at]].priority == priority {
            let root_index = ranked[index_at];
            searched.push(roots[root_index].name.clone());
            // 单根 loader 的未命中是常态，只有 escape 这类共享失败需要上报。
            if let Ok(id) = index.resolve(root_index, raw, None, &ext) {
                hits.push((root_index, id.path().to_string()));
            }
            index_at += 1;
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
        let Some(document) = self.catalog.document(&hit)
        else {
            return Err(IrError::Diagnostic(
                load_diag(codes::TEMPLATE_NOT_FOUND, ref_).arg("reason", "document-missing").arg("path", &hit.path),
            ));
        };
        let Some(id) = self.catalog.canonical_id(&hit)
        else {
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

/// 共享 loader 的失败：escape 保留为冻结码，其余按「根内未命中」处理。
fn shared_or_missing(
    error: document_template::TemplateError,
    reference: &str,
    from: Option<&str>,
    searched: Vec<String>,
) -> Diagnostic {
    if error.code() == shared::RELATIVE_ESCAPE_ROOT {
        let mut diag = load_diag(codes::TEMPLATE_RELATIVE_ESCAPE_ROOT, reference);
        if let Some(from) = from {
            diag = diag.arg("from", from);
        }
        return diag;
    }
    not_found(reference, from, searched)
}

/// 同一 priority 下多个 root 命中同一 bare 引用：拒绝按 catalog 顺序静默取胜。
fn ambiguous_ref(
    reference: &str,
    from: Option<&str>,
    priority: i32,
    hits: &[(usize, String)],
    roots: &[ResolveRoot],
) -> Diagnostic {
    let names: Vec<crate::ArgValue> =
        hits.iter().map(|(index, _)| crate::ArgValue::String(roots[*index].name.clone())).collect();
    let candidates: Vec<String> = hits.iter().map(|(index, path)| canonical_id(&roots[*index].name, path)).collect();
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

/// POSIX 归一化；越出 root 时返回 `Err`。
///
/// 保留为 compat 薄函数：只用于 `from` 这类 canonical id 的校验与归一，
/// 不参与引用解析（解析已交给共享 loader）。
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
