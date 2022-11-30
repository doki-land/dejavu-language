//! Translation validator for DejaVu Engine

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use super::{I18nConfig, extractor::TranslationItem};

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error type
    pub error_type: ValidationErrorType,
    /// Error message
    pub message: String,
    /// Related file path (if applicable)
    pub file_path: Option<String>,
}

/// Validation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationErrorType {
    /// Missing translation file for a locale
    MissingTranslationFile,
    /// Missing translation key in a locale
    MissingTranslationKey,
    /// Unused translation key
    UnusedTranslationKey,
    /// Invalid translation format
    InvalidTranslationFormat,
    /// Other error
    Other,
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub passed: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// List of validation warnings
    pub warnings: Vec<ValidationError>,
    /// Summary statistics
    pub stats: ValidationStats,
}

/// Validation statistics
#[derive(Debug, Clone)]
pub struct ValidationStats {
    /// Total number of translation keys
    pub total_keys: usize,
    /// Number of missing translations
    pub missing_translations: usize,
    /// Number of unused translations
    pub unused_translations: usize,
    /// Number of invalid translations
    pub invalid_translations: usize,
    /// Number of supported locales
    pub supported_locales: usize,
}

/// Translation validator
pub struct Validator {
    config: I18nConfig,
    extracted_translations: HashMap<String, TranslationItem>,
    locale_translations: HashMap<String, HashMap<String, String>>,
}

impl Validator {
    /// Create a new validator
    pub fn new(config: I18nConfig, extracted_translations: HashMap<String, TranslationItem>) -> Self {
        Self { config, extracted_translations, locale_translations: HashMap::new() }
    }

    /// Load translation files
    pub fn load_translations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create translations directory if it doesn't exist
        if !Path::new(&self.config.translations_dir).exists() {
            std::fs::create_dir_all(&self.config.translations_dir)?;
        }

        // Load translations for each supported locale
        for locale in &self.config.supported_locales {
            let translation_file = format!("{}/{}.po", self.config.translations_dir, locale);
            if Path::new(&translation_file).exists() {
                let translations = self.load_po_file(&translation_file)?;
                self.locale_translations.insert(locale.to_string(), translations);
            }
        }

        Ok(())
    }

    /// Load a PO file
    fn load_po_file(&self, file_path: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let mut translations = HashMap::new();
        let mut current_msgid = None;
        let mut current_msgstr = None;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("msgid ") {
                current_msgid = Some(line[6..].trim_matches('"').to_string());
                current_msgstr = None;
            } else if line.starts_with("msgstr ") {
                current_msgstr = Some(line[7..].trim_matches('"').to_string());
            }

            if let (Some(msgid), Some(msgstr)) = (current_msgid.clone(), current_msgstr.clone()) {
                if !msgid.is_empty() {
                    translations.insert(msgid, msgstr);
                }
                current_msgid = None;
                current_msgstr = None;
            }
        }

        Ok(translations)
    }

    /// Validate translations
    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut stats = ValidationStats {
            total_keys: self.extracted_translations.len(),
            missing_translations: 0,
            unused_translations: 0,
            invalid_translations: 0,
            supported_locales: self.config.supported_locales.len(),
        };

        // Check for missing translation files
        for locale in &self.config.supported_locales {
            let translation_file = format!("{}/{}.po", self.config.translations_dir, locale);
            if !Path::new(&translation_file).exists() {
                errors.push(ValidationError {
                    error_type: ValidationErrorType::MissingTranslationFile,
                    message: format!("Missing translation file for locale: {}", locale),
                    file_path: Some(translation_file),
                });
            }
        }

        // Check for missing translation keys in each locale
        let extracted_keys: HashSet<String> = self.extracted_translations.keys().cloned().collect();

        for (locale, translations) in &self.locale_translations {
            let locale_keys: HashSet<String> = translations.keys().cloned().collect();

            // Check for missing keys
            for key in &extracted_keys {
                if !locale_keys.contains(key) {
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::MissingTranslationKey,
                        message: format!("Missing translation key '{}' in locale {}", key, locale),
                        file_path: Some(format!("{}/{}.po", self.config.translations_dir, locale)),
                    });
                    stats.missing_translations += 1;
                }
            }

            // Check for unused keys
            for key in &locale_keys {
                if !extracted_keys.contains(key) {
                    warnings.push(ValidationError {
                        error_type: ValidationErrorType::UnusedTranslationKey,
                        message: format!("Unused translation key '{}' in locale {}", key, locale),
                        file_path: Some(format!("{}/{}.po", self.config.translations_dir, locale)),
                    });
                    stats.unused_translations += 1;
                }
            }
        }

        // Check for invalid translations (empty strings, etc.)
        for (locale, translations) in &self.locale_translations {
            for (key, value) in translations {
                if value.is_empty() {
                    errors.push(ValidationError {
                        error_type: ValidationErrorType::InvalidTranslationFormat,
                        message: format!("Empty translation for key '{}' in locale {}", key, locale),
                        file_path: Some(format!("{}/{}.po", self.config.translations_dir, locale)),
                    });
                    stats.invalid_translations += 1;
                }
            }
        }

        ValidationResult { passed: errors.is_empty(), errors, warnings, stats }
    }

    /// Generate validation report
    pub fn generate_report(&self, result: &ValidationResult) -> String {
        let mut report = String::new();

        report.push_str("# DejaVu Engine Translation Validation Report\n\n");

        // Summary
        report.push_str("## Summary\n");
        report.push_str(&format!("Total translation keys: {}\n", result.stats.total_keys));
        report.push_str(&format!("Supported locales: {}\n", result.stats.supported_locales));
        report.push_str(&format!("Missing translations: {}\n", result.stats.missing_translations));
        report.push_str(&format!("Unused translations: {}\n", result.stats.unused_translations));
        report.push_str(&format!("Invalid translations: {}\n", result.stats.invalid_translations));

        let status = if result.passed { "PASSED" } else { "FAILED" };
        report.push_str(&format!("Validation status: {}\n\n", status));

        // Errors
        if !result.errors.is_empty() {
            report.push_str("## Errors\n");
            for error in &result.errors {
                report.push_str(&format!("- [{}] {}", self.error_type_to_string(&error.error_type), error.message));
                if let Some(file_path) = &error.file_path {
                    report.push_str(&format!(" ({});", file_path));
                }
                report.push_str("\n");
            }
            report.push_str("\n");
        }

        // Warnings
        if !result.warnings.is_empty() {
            report.push_str("## Warnings\n");
            for warning in &result.warnings {
                report.push_str(&format!("- [{}] {}", self.error_type_to_string(&warning.error_type), warning.message));
                if let Some(file_path) = &warning.file_path {
                    report.push_str(&format!(" ({});", file_path));
                }
                report.push_str("\n");
            }
            report.push_str("\n");
        }

        report
    }

    /// Convert error type to string
    fn error_type_to_string(&self, error_type: &ValidationErrorType) -> &'static str {
        match error_type {
            ValidationErrorType::MissingTranslationFile => "MISSING_FILE",
            ValidationErrorType::MissingTranslationKey => "MISSING_KEY",
            ValidationErrorType::UnusedTranslationKey => "UNUSED_KEY",
            ValidationErrorType::InvalidTranslationFormat => "INVALID_FORMAT",
            ValidationErrorType::Other => "OTHER",
        }
    }
}
