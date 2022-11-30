//! Internationalization tools for DejaVu Engine

pub mod cli;
pub mod extractor;
pub mod ide;
pub mod validator;

/// I18n tool configuration
#[derive(Debug, Default, Clone)]
pub struct I18nConfig {
    /// Source directories to search for translatable content
    pub source_dirs: Vec<String>,
    /// Translation files directory
    pub translations_dir: String,
    /// Default locale
    pub default_locale: String,
    /// Supported locales
    pub supported_locales: Vec<String>,
    /// File patterns to include
    pub include_patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
}

impl I18nConfig {
    /// Create a new I18n config
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a source directory
    pub fn add_source_dir(&mut self, dir: &str) {
        self.source_dirs.push(dir.to_string());
    }

    /// Set translations directory
    pub fn set_translations_dir(&mut self, dir: &str) {
        self.translations_dir = dir.to_string();
    }

    /// Set default locale
    pub fn set_default_locale(&mut self, locale: &str) {
        self.default_locale = locale.to_string();
    }

    /// Add a supported locale
    pub fn add_supported_locale(&mut self, locale: &str) {
        self.supported_locales.push(locale.to_string());
    }

    /// Add an include pattern
    pub fn add_include_pattern(&mut self, pattern: &str) {
        self.include_patterns.push(pattern.to_string());
    }

    /// Add an exclude pattern
    pub fn add_exclude_pattern(&mut self, pattern: &str) {
        self.exclude_patterns.push(pattern.to_string());
    }
}
