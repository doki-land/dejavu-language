//! `document-template` 共享合同与 dejavu 冻结诊断之间的唯一适配层。
//!
//! 引用分类、相对路径拼接、扩展名探测与根内命中判定全部发生在共享
//! [`MapTemplateLoader`] 内；本模块只做三件事：
//! 1. 把 dejavu 的 root 索引翻译成「每根一个 [`MapTemplateLoader`]」；
//! 2. 在 `TemplateId`（`root:path`）与 dejavu 的 canonical id 字符串之间互转；
//! 3. 把共享稳定码翻译为 dejavu 冻结码，并补回 dejavu 的诊断参数形状。
//!
//! 多根优先级的歧义判定属于宿主策略（dejavu 边界），共享 loader 只负责单根。

#![allow(dead_code)]

use dejavu_diagnostic::{Diagnostic, Severity, Stage};
use document_template::{MapTemplateLoader, TemplateError, TemplateId, TemplateLoader, TemplateRoot, codes as shared};

use crate::catalog::ResolveRoot;

/// dejavu root 索引对应的共享 loader 集合：下标与 [`ResolveRoot`] 一一对应。
pub(crate) struct RootIndex {
    loaders: Vec<MapTemplateLoader>,
}

impl RootIndex {
    /// 按 dejavu 的 root 顺序构造单根 loader。
    ///
    /// 共享 loader 只需要「路径存在」这一事实，因此文件内容登记为空串；
    /// dejavu 的 IR 文档仍留在 [`crate::TemplateCatalog`]。
    pub(crate) fn new(roots: &[ResolveRoot]) -> Self {
        let loaders = roots
            .iter()
            .map(|root| {
                let mut template_root = TemplateRoot::new(root.name.clone(), 0);
                for file in &root.files {
                    template_root = template_root.with_file(file.clone(), "");
                }
                MapTemplateLoader::new().with_root(template_root)
            })
            .collect();
        Self { loaders }
    }

    /// 在指定根内解析一次引用。
    pub(crate) fn resolve(
        &self,
        index: usize,
        reference: &str,
        from: Option<&TemplateId>,
        extensions: &[&str],
    ) -> Result<TemplateId, TemplateError> {
        self.loaders[index].resolve(reference, from, extensions)
    }

    /// 根数量。
    pub(crate) fn len(&self) -> usize {
        self.loaders.len()
    }
}

/// `TemplateId` → dejavu canonical id（`root:path`）。
pub(crate) fn canonical_id(id: &TemplateId) -> String {
    format!("{}:{}", id.root(), id.path())
}

/// 共享稳定码 → dejavu 冻结码。
///
/// 只登记 dejavu 边界真会收到的码；未登记返回 `None`，由调用方保留原码，
/// 避免静默改写。
pub(crate) fn mapped_code(shared_code: &str) -> Option<&'static str> {
    match shared_code {
        shared::INVALID_REFERENCE => Some(dejavu_diagnostic::codes::TEMPLATE_INVALID_REF),
        shared::UNKNOWN_SCHEME => Some(dejavu_diagnostic::codes::TEMPLATE_UNKNOWN_SCHEME),
        shared::RELATIVE_WITHOUT_FROM => Some(dejavu_diagnostic::codes::TEMPLATE_RELATIVE_WITHOUT_FROM),
        shared::RELATIVE_ESCAPE_ROOT => Some(dejavu_diagnostic::codes::TEMPLATE_RELATIVE_ESCAPE_ROOT),
        shared::NOT_FOUND => Some(dejavu_diagnostic::codes::TEMPLATE_NOT_FOUND),
        _ => None,
    }
}

/// 把共享 loader 的失败翻译成 dejavu load 阶段诊断。
///
/// `reference` 与 `from` 由边界传入，因为 dejavu 的 `args` 形状是冻结的
/// （例如 `searched` 必须是数组），不能直接照搬共享的散文式参数。
pub(crate) fn load_diagnostic(error: &TemplateError, reference: &str, from: Option<&str>) -> Diagnostic {
    let code = mapped_code(error.code()).map_or_else(|| error.code().to_string(), str::to_string);
    let diagnostic = Diagnostic::new(Severity::Error, code, Stage::Load).arg("ref", reference);
    match (error.code(), from) {
        (shared::RELATIVE_ESCAPE_ROOT, Some(from)) => diagnostic.arg("from", from),
        _ => diagnostic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(name: &str, files: &[&str]) -> ResolveRoot {
        ResolveRoot {
            name: name.into(),
            scheme: name.into(),
            priority: 0,
            files: files.iter().map(|file| (*file).to_string()).collect(),
        }
    }

    fn extensions() -> Vec<&'static str> {
        vec!["", ".html", ".dejavu"]
    }

    #[test]
    fn per_root_loader_probes_extensions() {
        let roots = vec![root("pages", &["index.dejavu", "partials/nav.dejavu"])];
        let index = RootIndex::new(&roots);
        let hit = index.resolve(0, "index", None, &extensions()).unwrap();
        assert_eq!(canonical_id(&hit), "pages:index.dejavu");
    }

    #[test]
    fn per_root_loader_stays_inside_its_root() {
        let roots = vec![root("app", &["index.dejavu"]), root("theme", &["layout.dejavu"])];
        let index = RootIndex::new(&roots);
        assert!(index.resolve(0, "layout", None, &extensions()).is_err());
        assert_eq!(index.resolve(1, "layout", None, &extensions()).unwrap().path(), "layout.dejavu");
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn relative_reference_uses_from_id() {
        let roots = vec![root("pages", &["a/b/child.dejavu", "a/shared.dejavu"])];
        let index = RootIndex::new(&roots);
        let from = TemplateId::new("pages", "a/b/child.dejavu");
        let hit = index.resolve(0, "../shared", Some(&from), &extensions()).unwrap();
        assert_eq!(canonical_id(&hit), "pages:a/shared.dejavu");
    }

    #[test]
    fn shared_codes_map_to_frozen_dejavu_codes() {
        assert_eq!(mapped_code(shared::NOT_FOUND), Some(dejavu_diagnostic::codes::TEMPLATE_NOT_FOUND));
        assert_eq!(mapped_code(shared::UNKNOWN_SCHEME), Some(dejavu_diagnostic::codes::TEMPLATE_UNKNOWN_SCHEME));
        assert_eq!(mapped_code("template.step-budget"), None);
    }

    #[test]
    fn loader_failure_keeps_frozen_code_and_reference() {
        let roots = vec![root("pages", &["index.dejavu"])];
        let index = RootIndex::new(&roots);
        let error = index.resolve(0, "nope", None, &extensions()).unwrap_err();
        let diagnostic = load_diagnostic(&error, "nope", None);
        assert_eq!(diagnostic.code, dejavu_diagnostic::codes::TEMPLATE_NOT_FOUND);
        assert_eq!(diagnostic.args.get("ref").cloned(), Some("nope".into()));
        assert_eq!(diagnostic.stage, Stage::Load);
    }
}
