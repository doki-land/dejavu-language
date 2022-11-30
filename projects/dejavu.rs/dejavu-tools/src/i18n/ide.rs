//! IDE integration for DejaVu Engine internationalization tools

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::{I18nConfig, extractor::Extractor, validator::Validator};

/// IDE integration manager
pub struct IdeIntegration {
    config: I18nConfig,
    extracted_translations: HashMap<String, String>,
}

impl IdeIntegration {
    /// Create a new IDE integration manager
    pub fn new(config: I18nConfig) -> Self {
        Self { config, extracted_translations: HashMap::new() }
    }

    /// Initialize IDE integration
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Extract translations to populate the cache
        let mut extractor = Extractor::new(self.config.clone());
        let extracted = extractor.extract()?;

        // Convert to simple key-value map for IDE use
        for (key, item) in extracted {
            self.extracted_translations.insert(key, item.source);
        }

        Ok(())
    }

    /// Get translation suggestions for a key
    pub fn get_translation_suggestions(&self, key: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Exact match
        if let Some(source) = self.extracted_translations.get(key) {
            suggestions.push(source.clone());
        }

        // Partial matches
        for (k, v) in &self.extracted_translations {
            if k.contains(key) && k != key {
                suggestions.push(format!("{}: {}", k, v));
            }
        }

        suggestions
    }

    /// Validate translations for a specific file
    pub fn validate_file(&self, file_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut errors = Vec::new();

        // Create a temporary config for this file
        let mut file_config = self.config.clone();
        file_config.source_dirs = vec![Path::new(file_path).parent().unwrap().to_string_lossy().to_string()];

        // Extract translations from this file
        let mut extractor = Extractor::new(file_config.clone());
        let extracted = extractor.extract()?;

        // Validate the extracted translations
        let mut validator = Validator::new(file_config, extracted);
        validator.load_translations()?;

        let result = validator.validate();

        // Convert validation errors to IDE-friendly messages
        for error in result.errors {
            errors.push(error.message);
        }

        for warning in result.warnings {
            errors.push(format!("Warning: {}", warning.message));
        }

        Ok(errors)
    }

    /// Get all translation keys
    pub fn get_all_translation_keys(&self) -> Vec<String> {
        self.extracted_translations.keys().cloned().collect()
    }

    /// Get translation for a key
    pub fn get_translation(&self, key: &str) -> Option<&String> {
        self.extracted_translations.get(key)
    }

    /// Check if a translation key exists
    pub fn has_translation(&self, key: &str) -> bool {
        self.extracted_translations.contains_key(key)
    }

    /// Refresh translations cache
    pub fn refresh_translations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.initialize()
    }
}

/// LSP (Language Server Protocol) support
pub struct I18nLanguageServer {
    ide_integration: IdeIntegration,
}

impl I18nLanguageServer {
    /// Create a new language server
    pub fn new(config: I18nConfig) -> Self {
        Self { ide_integration: IdeIntegration::new(config) }
    }

    /// Initialize the language server
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.ide_integration.initialize()
    }

    /// Handle text document completion
    pub fn handle_completion(&self, text: &str, position: (usize, usize)) -> Vec<String> {
        // Extract the current word being typed
        let lines: Vec<&str> = text.lines().collect();
        if position.0 >= lines.len() {
            return Vec::new();
        }

        let current_line = lines[position.0];
        let line_before_cursor = &current_line[..position.1];

        // Look for t() function calls
        if let Some(open_paren) = line_before_cursor.rfind('(') {
            let after_open_paren = &line_before_cursor[open_paren + 1..];
            if after_open_paren.trim().starts_with('"') || after_open_paren.trim().starts_with('\'') {
                // Extract the partial key
                let quote = after_open_paren.trim().chars().next().unwrap();
                let key_start = after_open_paren.trim().find(quote).unwrap() + 1;
                let partial_key = if let Some(quote_end) = after_open_paren.trim()[key_start..].find(quote) {
                    &after_open_paren.trim()[key_start..key_start + quote_end]
                } else {
                    &after_open_paren.trim()[key_start..]
                };

                return self.ide_integration.get_translation_suggestions(partial_key);
            }
        }

        Vec::new()
    }

    /// Handle text document diagnostics
    pub fn handle_diagnostics(&self, file_path: &str, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Validate the file
        if let Ok(errors) = self.ide_integration.validate_file(file_path) {
            for (line_num, line) in content.lines().enumerate() {
                // Look for t() function calls
                let mut char_pos = 0;
                while let Some(t_pos) = line[char_pos..].find("t(") {
                    let t_start = char_pos + t_pos;
                    char_pos = t_start + 2;

                    // Extract the key
                    if let Some(quote) = line[char_pos..].chars().find(|c| *c == '"' || *c == '\'') {
                        let quote_pos = line[char_pos..].find(quote).unwrap();
                        let key_start = char_pos + quote_pos + 1;
                        if let Some(key_end) = line[key_start..].find(quote) {
                            let key = &line[key_start..key_start + key_end];

                            // Check if key exists
                            if !self.ide_integration.has_translation(key) {
                                diagnostics.push(Diagnostic {
                                    range: (line_num, t_start, line_num, t_start + 2 + quote_pos + 1 + key_end + 1),
                                    message: format!("Translation key '{}' not found", key),
                                    severity: DiagnosticSeverity::Warning,
                                });
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// Diagnostic information for IDE
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Range (start line, start char, end line, end char)
    pub range: (usize, usize, usize, usize),
    /// Diagnostic message
    pub message: String,
    /// Diagnostic severity
    pub severity: DiagnosticSeverity,
}

/// Diagnostic severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Error
    Error,
    /// Warning
    Warning,
    /// Information
    Information,
    /// Hint
    Hint,
}
