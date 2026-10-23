//! 扫描目录下所有匹配扩展名的模板文件。

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// N-API 默认扩展名（dejavu-napi 默认识别的模板后缀）。
/// 空字符串表示"无扩展名的纯模板文件"也匹配。
pub const DEFAULT_EXTENSIONS: &[&str] = &["", ".dejavu", ".html", ".doki"];

/// 默认排除的目录名。
const DEFAULT_EXCLUDES: &[&str] = &[".git", "node_modules"];

/// 扫描配置。
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// 要匹配的扩展名列表。空字符串表示"无扩展名也匹配"。
    pub extensions: Vec<String>,
    /// 要排除的目录名。
    pub excludes: Vec<String>,
    /// 不跟随符号链接（默认 true）。
    pub follow_links: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            extensions: DEFAULT_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
            excludes: DEFAULT_EXCLUDES.iter().map(|s| s.to_string()).collect(),
            follow_links: false,
        }
    }
}

impl ScanConfig {
    /// 从逗号分隔的扩展名字符串构造（例如 `"".dejavu,.html,.doki"`）。
    pub fn with_extensions_csv(mut self, csv: &str) -> Self {
        self.extensions = csv
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() || csv.split(',').any(|x| x.trim().is_empty()))
            .collect();
        // 如果用户传了空字符串，保留它；如果什么都没传，回退到默认
        if self.extensions.is_empty() {
            self.extensions = DEFAULT_EXTENSIONS.iter().map(|s| s.to_string()).collect();
        }
        self
    }

    /// 额外排除的目录（相对于 source 根的顶层目录名）。
    pub fn exclude(mut self, dir: impl Into<String>) -> Self {
        self.excludes.push(dir.into());
        self
    }

    fn matches_extension(&self, path: &Path) -> bool {
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => return false,
        };

        // 空扩展名项：匹配完全没有扩展名的文件（不含 `.` 或者只有前导点）
        let has_extension = file_name.contains('.');
        for ext in &self.extensions {
            if ext.is_empty() {
                if !has_extension {
                    return true;
                }
                continue;
            }
            if file_name.ends_with(ext) {
                return true;
            }
        }
        false
    }
}

/// 扫描目录，返回匹配的模板文件路径列表（按路径字典序排序）。
///
/// * `source` — 要扫描的目录。
/// * `target` — 可选的目标目录路径，扫描时排除（通常是 outDir）。
///
/// 错误情况：source 不存在或不是目录时返回 Err。
pub fn scan_templates(
    source: &Path,
    config: &ScanConfig,
    target: Option<&Path>,
) -> Result<Vec<PathBuf>, walkdir::Error> {
    let mut walker = WalkDir::new(source)
        .follow_links(config.follow_links)
        .into_iter();

    let mut results: Vec<PathBuf> = Vec::new();

    while let Some(entry) = walker.next() {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                // 跳过我们要排除的目录时产生的错误继续；根目录错误向上传播
                if err.depth() > 0 {
                    continue;
                }
                return Err(err);
            }
        };

        // 跳过目录本身，但继续扫描其子项
        if entry.file_type().is_dir() {
            let name = entry.file_name().to_string_lossy();
            // 排除目标目录自身
            if let Some(tgt) = target {
                if entry.path() == tgt {
                    walker.skip_current_dir();
                    continue;
                }
            }
            if config.excludes.iter().any(|ex| ex == &name) {
                walker.skip_current_dir();
                continue;
            }
            continue;
        }

        // 只关心普通文件（不跟随符号链接，所以 symlink 到文件的条目会被 WalkDir 按 follow_links=false 处理）
        if !entry.file_type().is_file() {
            continue;
        }

        if config.matches_extension(entry.path()) {
            results.push(entry.into_path());
        }
    }

    results.sort();
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // 为测试手动创建临时目录（避免再加 dev-dependency）
    struct TestRoot {
        path: PathBuf,
    }
    impl TestRoot {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "dejavu-tools-test-{}-{}",
                std::process::id(),
                tag
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self { path: dir }
        }
        fn mk(&self, rel: &str) -> PathBuf {
            let p = self.path.join(rel);
            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&p, "").unwrap();
            p
        }
        fn dir(&self, rel: &str) -> PathBuf {
            let p = self.path.join(rel);
            fs::create_dir_all(&p).unwrap();
            p
        }
    }
    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn scan_finds_default_extensions() {
        let root = TestRoot::new("defaults");
        root.mk("a.dejavu");
        root.mk("b.html");
        root.mk("c.doki");
        root.mk("d");          // 无扩展名 — 应该被默认扩展名中的 "" 匹配
        root.mk("e.txt");      // 不匹配
        root.mk("sub/nested.dejavu");
        root.dir(".git");
        root.mk(".git/should_not_appear.dejavu");
        root.dir("node_modules");
        root.mk("node_modules/should_not_appear.html");

        let config = ScanConfig::default();
        let found = scan_templates(&root.path, &config, None).unwrap();

        let names: Vec<String> = found
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(names.contains(&"a.dejavu".to_string()));
        assert!(names.contains(&"b.html".to_string()));
        assert!(names.contains(&"c.doki".to_string()));
        assert!(names.contains(&"d".to_string()));
        assert!(!names.contains(&"e.txt".to_string()));
        assert!(names.contains(&"nested.dejavu".to_string()));
        // .git 和 node_modules 被排除
        assert!(!found.iter().any(|p| p.to_string_lossy().contains(".git")));
        assert!(!found.iter().any(|p| p.to_string_lossy().contains("node_modules")));
    }

    #[test]
    fn scan_respects_extensions_override() {
        let root = TestRoot::new("override");
        root.mk("a.dejavu");
        root.mk("b.html");
        root.mk("c.custom");

        let config = ScanConfig::default().with_extensions_csv(".dejavu,.custom");
        let found = scan_templates(&root.path, &config, None).unwrap();

        let names: Vec<String> = found
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(names.contains(&"a.dejavu".to_string()));
        assert!(!names.contains(&"b.html".to_string()));
        assert!(names.contains(&"c.custom".to_string()));
    }

    #[test]
    fn scan_excludes_target_dir() {
        let root = TestRoot::new("target");
        root.mk("a.dejavu");
        let target = root.dir("dist");
        root.mk("dist/compiled.dejavu");

        let config = ScanConfig::default();
        let found = scan_templates(&root.path, &config, Some(&target)).unwrap();

        assert!(!found.iter().any(|p| p.starts_with(&target)));
    }
}
