//! `document-template` 共享合同与 dejavu 宿主之间的唯一适配层。
//!
//! 引用分类、相对路径拼接、扩展名探测与根内命中判定全部发生在共享
//! [`MapTemplateLoader`] 内；本模块只把 dejavu 的 root 索引翻译成
//! 「每根一个 [`MapTemplateLoader`]」。
//!
//! 多根优先级与同优先级歧义属于宿主策略（dejavu 边界），共享 loader 只负责单根。

use document_template::{MapTemplateLoader, TemplateError, TemplateId, TemplateLoader, TemplateRoot};

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
        assert_eq!(format!("{}:{}", hit.root(), hit.path()), "pages:index.dejavu");
    }

    #[test]
    fn per_root_loader_stays_inside_its_root() {
        let roots = vec![root("app", &["index.dejavu"]), root("theme", &["layout.dejavu"])];
        let index = RootIndex::new(&roots);
        assert!(index.resolve(0, "layout", None, &extensions()).is_err());
        assert_eq!(index.resolve(1, "layout", None, &extensions()).unwrap().path(), "layout.dejavu");
    }

    #[test]
    fn relative_reference_uses_from_id() {
        let roots = vec![root("pages", &["a/b/child.dejavu", "a/shared.dejavu"])];
        let index = RootIndex::new(&roots);
        let from = TemplateId::new("pages", "a/b/child.dejavu");
        let hit = index.resolve(0, "../shared", Some(&from), &extensions()).unwrap();
        assert_eq!(format!("{}:{}", hit.root(), hit.path()), "pages:a/shared.dejavu");
    }
}
