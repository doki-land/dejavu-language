//! Translation extractor for DejaVu Engine

use glob::glob;
use regex::Regex;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use super::I18nConfig;

/// Extracted translation item
#[derive(Debug, Clone)]
pub struct TranslationItem {
    /// Translation key
    pub key: String,
    /// Source text
    pub source: String,
    /// File path where the translation was found
    pub file_path: String,
    /// Line number
    pub line_number: usize,
    /// Context information
    pub context: Option<String>,
}

/// Translation extractor
pub struct Extractor {
    config: I18nConfig,
    translations: HashMap<String, TranslationItem>,
}

impl Extractor {
    /// Create a new extractor
    pub fn new(config: I18nConfig) -> Self {
        Self { config, translations: HashMap::new() }
    }

    /// Extract translations from all source files
    pub fn extract(&mut self) -> Result<HashMap<String, TranslationItem>, Box<dyn std::error::Error>> {
        // Process each source directory
        let source_dirs = self.config.source_dirs.clone();
        for source_dir in source_dirs {
            self.extract_from_dir(&source_dir)?;
        }

        Ok(self.translations.clone())
    }

    /// Extract translations from a directory
    fn extract_from_dir(&mut self, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        let patterns = if self.config.include_patterns.is_empty() {
            vec!["**/*.dejavu".to_string(), "**/*.rs".to_string(), "**/*.ts".to_string(), "**/*.js".to_string()]
        } else {
            self.config.include_patterns.clone()
        };

        for pattern in patterns {
            let full_pattern = format!("{}/{}", dir, pattern);
            for entry in glob(&full_pattern)? {
                match entry {
                    Ok(path) => {
                        // Check if file should be excluded
                        if self.should_exclude(&path) {
                            continue;
                        }

                        self.extract_from_file(&path)?;
                    }
                    Err(e) => println!("Error globbing: {:?}", e),
                }
            }
        }

        Ok(())
    }

    /// Check if a file should be excluded
    fn should_exclude(&self, path: &Path) -> bool {
        for pattern in &self.config.exclude_patterns {
            if let Ok(matched) = glob::Pattern::new(pattern).and_then(|p| Ok(p.matches_path(path))) {
                if matched {
                    return true;
                }
            }
        }
        false
    }

    /// Extract translations from a file
    fn extract_from_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let file_path = path.to_string_lossy().to_string();

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("dejavu") => {
                self.extract_from_dejavu_template(&content, &file_path)?;
            }
            Some("rs") => {
                self.extract_from_rust_file(&content, &file_path)?;
            }
            Some("ts") | Some("js") => {
                self.extract_from_js_file(&content, &file_path)?;
            }
            _ => {
                // Skip other file types
            }
        }

        Ok(())
    }

    /// Extract translations from DejaVu template
    fn extract_from_dejavu_template(&mut self, content: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Regex to match t() function calls in templates
        let t_function_regex = Regex::new(r#"t\s*\(\s*["']([^"']+)["']"#)?;

        for (line_num, line) in content.lines().enumerate() {
            for capture in t_function_regex.captures_iter(line) {
                if let Some(key_str) = capture.get(1) {
                    let key = key_str.as_str().to_string();
                    let item = TranslationItem {
                        key: key.clone(),
                        source: key,
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                        context: None,
                    };
                    self.translations.insert(key_str.as_str().to_string(), item);
                }
            }
        }

        Ok(())
    }

    /// Extract translations from Rust file
    fn extract_from_rust_file(&mut self, content: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Regex to match t() function calls in Rust
        let t_function_regex = Regex::new(r#"t\s*\(\s*["']([^"']+)["']"#)?;

        for (line_num, line) in content.lines().enumerate() {
            for capture in t_function_regex.captures_iter(line) {
                if let Some(key_str) = capture.get(1) {
                    let key = key_str.as_str().to_string();
                    let item = TranslationItem {
                        key: key.clone(),
                        source: key,
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                        context: None,
                    };
                    self.translations.insert(key_str.as_str().to_string(), item);
                }
            }
        }

        Ok(())
    }

    /// Extract translations from JavaScript/TypeScript file
    fn extract_from_js_file(&mut self, content: &str, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Regex to match t() function calls in JS/TS
        let t_function_regex = Regex::new(r#"t\s*\(\s*["']([^"']+)["']"#)?;

        for (line_num, line) in content.lines().enumerate() {
            for capture in t_function_regex.captures_iter(line) {
                if let Some(key_str) = capture.get(1) {
                    let key = key_str.as_str().to_string();
                    let item = TranslationItem {
                        key: key.clone(),
                        source: key,
                        file_path: file_path.to_string(),
                        line_number: line_num + 1,
                        context: None,
                    };
                    self.translations.insert(key_str.as_str().to_string(), item);
                }
            }
        }

        Ok(())
    }

    /// Save extracted translations to a file
    pub fn save_extracted(&self, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut content = String::new();

        for (key, item) in &self.translations {
            content.push_str(&format!("#: {}:{}\n", item.file_path, item.line_number));
            content.push_str(&format!("msgid \"{}\"\n", key));
            content.push_str(&format!("msgstr \"{}\"\n\n", item.source));
        }

        fs::write(output_path, content)?;
        Ok(())
    }
}
