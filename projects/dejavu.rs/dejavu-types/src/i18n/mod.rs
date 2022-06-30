//! Internationalization support for Dejavu templates

use crate::DejavuResult;
use fluent_bundle::{FluentBundle, FluentResource};
use lru::LruCache;
use std::{collections::HashMap, fmt::Debug, fs, hash::Hash, path::Path, time::SystemTime};
use unic_langid::LanguageIdentifier;

/// Translation provider trait for internationalization
pub trait TranslationProvider: AsAny + Debug {
    /// Translate a key with optional arguments
    ///
    /// # Arguments
    /// * `key` - Translation key
    /// * `args` - Optional translation arguments
    /// * `locale` - Locale identifier
    ///
    /// # Returns
    /// Translated string or the key if translation not found
    fn translate(&self, key: &str, args: Option<&HashMap<String, String>>, locale: &str) -> DejavuResult<String>;

    /// Get the current locale
    fn get_locale(&self) -> &str;

    /// Set the current locale
    fn set_locale(&mut self, locale: &str);

    /// Clone the translator
    fn clone_box(&self) -> Box<dyn TranslationProvider>;
}

/// Simple translation data structure
#[derive(Debug, Clone)]
pub struct TranslationData {
    /// Available translations
    pub translations: HashMap<String, HashMap<String, String>>,
    /// Current locale
    pub current_locale: String,
    /// Default locale (fallback)
    pub default_locale: String,
}

impl TranslationData {
    /// Create a new translation data instance
    pub fn new(current_locale: &str, default_locale: &str) -> Self {
        Self {
            translations: HashMap::new(),
            current_locale: current_locale.to_string(),
            default_locale: default_locale.to_string(),
        }
    }

    /// Add translations for a locale
    pub fn add_translations(&mut self, locale: &str, translations: HashMap<String, String>) {
        self.translations.insert(locale.to_string(), translations);
    }

    /// Get translation for a key
    pub fn get_translation(&self, key: &str, locale: &str) -> Option<&String> {
        // Try current locale
        if let Some(lang_translations) = self.translations.get(locale) {
            if let Some(translation) = lang_translations.get(key) {
                return Some(translation);
            }
        }

        // Try default locale as fallback
        if locale != self.default_locale {
            if let Some(lang_translations) = self.translations.get(&self.default_locale) {
                if let Some(translation) = lang_translations.get(key) {
                    return Some(translation);
                }
            }
        }

        None
    }
}

/// Simple implementation of TranslationProvider
#[derive(Debug, Clone)]
pub struct SimpleTranslator {
    /// Translation data
    pub data: TranslationData,
}

impl SimpleTranslator {
    /// Create a new simple translator
    pub fn new(current_locale: &str, default_locale: &str) -> Self {
        Self { data: TranslationData::new(current_locale, default_locale) }
    }

    /// Add translations for a locale
    pub fn add_translations(&mut self, locale: &str, translations: HashMap<String, String>) {
        self.data.add_translations(locale, translations);
    }
}

impl TranslationProvider for SimpleTranslator {
    fn translate(&self, key: &str, args: Option<&HashMap<String, String>>, locale: &str) -> DejavuResult<String> {
        let translation = match self.data.get_translation(key, locale) {
            Some(t) => t.to_string(),
            None => key.to_string(),
        };

        // Handle placeholders if args are provided
        if let Some(args) = args {
            let mut result = translation;
            for (name, value) in args {
                result = result.replace(&format!("{{{}}}", name), value);
            }
            Ok(result)
        } else {
            Ok(translation)
        }
    }

    fn get_locale(&self) -> &str {
        &self.data.current_locale
    }

    fn set_locale(&mut self, locale: &str) {
        self.data.current_locale = locale.to_string();
    }

    fn clone_box(&self) -> Box<dyn TranslationProvider> {
        Box::new(self.clone())
    }
}

/// Translation cache key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TranslationCacheKey {
    /// Translation key
    key: String,
    /// Locale
    locale: String,
    /// Arguments hash
    args_hash: u64,
}

impl TranslationCacheKey {
    /// Create a new cache key
    pub fn new(key: &str, locale: &str, args: Option<&HashMap<String, String>>) -> Self {
        // Generate a more robust hash for arguments
        let args_hash = match args {
            Some(args) => {
                use std::{collections::hash_map::DefaultHasher, hash::Hasher};

                let mut hasher = DefaultHasher::new();
                // Sort keys to ensure consistent hashing regardless of insertion order
                let mut sorted_keys: Vec<&String> = args.keys().collect();
                sorted_keys.sort();

                for k in sorted_keys {
                    k.hash(&mut hasher);
                    args.get(k).unwrap().hash(&mut hasher);
                }
                hasher.finish()
            }
            None => 0,
        };

        Self { key: key.to_string(), locale: locale.to_string(), args_hash }
    }
}

/// I18n context for runtime internationalization
#[derive(Debug)]
pub struct I18nContext {
    /// Translation provider
    translator: Box<dyn TranslationProvider>,
    /// Translation cache
    translation_cache: LruCache<TranslationCacheKey, String>,
    /// Default cache capacity
    cache_capacity: usize,
}

impl Clone for I18nContext {
    fn clone(&self) -> Self {
        Self {
            translator: self.translator.clone_box(),
            translation_cache: LruCache::new(self.cache_capacity),
            cache_capacity: self.cache_capacity,
        }
    }
}

impl I18nContext {
    /// Create a new I18n context
    pub fn new(translator: Box<dyn TranslationProvider>) -> Self {
        Self {
            translator,
            translation_cache: LruCache::new(1000), // Increased default capacity
            cache_capacity: 1000,
        }
    }

    /// Create a new I18n context with custom cache capacity
    pub fn with_cache_capacity(translator: Box<dyn TranslationProvider>, capacity: usize) -> Self {
        Self { translator, translation_cache: LruCache::new(capacity), cache_capacity: capacity }
    }

    /// Translate a key with optional arguments
    pub fn t(&mut self, key: &str, args: Option<&HashMap<String, String>>) -> DejavuResult<String> {
        let locale = self.translator.get_locale();
        let cache_key = TranslationCacheKey::new(key, locale, args);

        // Check cache first (fast path)
        if let Some(cached) = self.translation_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Translate and cache the result
        let result = self.translator.translate(key, args, locale)?;
        self.translation_cache.put(cache_key, result.clone());
        Ok(result)
    }

    /// Batch translate multiple keys at once
    pub fn t_batch(&mut self, keys: &[(String, Option<HashMap<String, String>>)]) -> DejavuResult<Vec<String>> {
        let locale = self.translator.get_locale();
        let mut results = Vec::with_capacity(keys.len());
        let mut missing = Vec::new();

        // First check cache for all keys
        for (key, args) in keys {
            let cache_key = TranslationCacheKey::new(key, locale, args.as_ref());
            if let Some(cached) = self.translation_cache.get(&cache_key) {
                results.push(cached.clone());
            } else {
                missing.push((key.clone(), args.clone()));
                results.push(String::new()); // Placeholder
            }
        }

        // Translate missing keys
        for (i, (key, args)) in keys.iter().enumerate() {
            if results[i].is_empty() {
                let result = self.translator.translate(key, args.as_ref(), locale)?;
                let cache_key = TranslationCacheKey::new(key, locale, args.as_ref());
                self.translation_cache.put(cache_key, result.clone());
                results[i] = result;
            }
        }

        Ok(results)
    }

    /// Get the current locale
    pub fn get_locale(&self) -> &str {
        self.translator.get_locale()
    }

    /// Set the current locale
    pub fn set_locale(&mut self, locale: &str) {
        self.translator.set_locale(locale);
        // Clear cache when locale changes
        self.translation_cache.clear();
    }

    /// Format a number according to the current locale
    pub fn format_number(&self, number: f64) -> String {
        // Simple implementation - in a real-world scenario, use a proper number formatting library
        let locale = self.translator.get_locale();
        match locale {
            "zh" | "zh-CN" | "zh-TW" => {
                // Chinese formatting
                format!("{:.2}", number).replace('.', "。")
            }
            _ => {
                // Default formatting
                format!("{:.2}", number)
            }
        }
    }

    /// Format a date according to the current locale
    pub fn format_date(&self, date: SystemTime) -> String {
        // Simple implementation - in a real-world scenario, use a proper date formatting library
        let locale = self.translator.get_locale();
        let now = SystemTime::now();
        let duration = now.duration_since(date).unwrap_or_default();
        let days = duration.as_secs() / (24 * 3600);

        match locale {
            "zh" | "zh-CN" | "zh-TW" => {
                if days == 0 {
                    "今天".to_string()
                } else if days == 1 {
                    "昨天".to_string()
                } else if days < 7 {
                    format!("{}天前", days)
                } else {
                    format!("{:?}", date)
                }
            }
            _ => {
                if days == 0 {
                    "Today".to_string()
                } else if days == 1 {
                    "Yesterday".to_string()
                } else if days < 7 {
                    format!("{} days ago", days)
                } else {
                    format!("{:?}", date)
                }
            }
        }
    }

    /// Add translations for a locale (only supported for SimpleTranslator)
    pub fn add_translations(&mut self, locale: &str, translations: HashMap<String, String>) -> DejavuResult<()> {
        // Clear cache when adding new translations
        self.translation_cache.clear();

        // Try to downcast to SimpleTranslator using AsAny trait
        if let Some(simple_translator) = AsAny::as_any_mut(&mut *self.translator).downcast_mut::<SimpleTranslator>() {
            simple_translator.add_translations(locale, translations);
            Ok(())
        } else {
            Err(crate::DejavuError::ParseError("Adding translations is only supported for SimpleTranslator".to_string()))
        }
    }

    /// Clear the translation cache
    pub fn clear_cache(&mut self) {
        self.translation_cache.clear();
    }

    /// Get current cache size
    pub fn cache_size(&self) -> usize {
        self.translation_cache.len()
    }
}

/// Helper trait to enable downcasting
use std::any::Any;

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Fluent translation provider implementation
pub struct FluentTranslator {
    /// Fluent bundles for different locales
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    /// Current locale
    current_locale: String,
    /// Default locale (fallback)
    default_locale: String,
}

impl Debug for FluentTranslator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FluentTranslator")
            .field("current_locale", &self.current_locale)
            .field("default_locale", &self.default_locale)
            .field("bundle_count", &self.bundles.len())
            .finish()
    }
}

impl FluentTranslator {
    /// Create a new Fluent translator
    pub fn new(current_locale: &str, default_locale: &str) -> Self {
        Self { bundles: HashMap::new(), current_locale: current_locale.to_string(), default_locale: default_locale.to_string() }
    }

    /// Add translations from a .ftl file for a locale
    pub fn add_translations_from_file(&mut self, locale: &str, file_path: &Path) -> DejavuResult<()> {
        let ftl_content = fs::read_to_string(file_path)?;
        self.add_translations_from_string(locale, &ftl_content)
    }

    /// Add translations from a string for a locale
    pub fn add_translations_from_string(&mut self, locale: &str, ftl_content: &str) -> DejavuResult<()> {
        let resource = match FluentResource::try_new(ftl_content.to_string()) {
            Ok(resource) => resource,
            Err((_, errors)) => {
                return Err(crate::DejavuError::ParseError(format!("Failed to create Fluent resource: {:?}", errors)));
            }
        };

        let lang_id: LanguageIdentifier = match locale.parse() {
            Ok(id) => id,
            Err(err) => return Err(crate::DejavuError::ParseError(format!("Invalid locale identifier: {:?}", err))),
        };

        let mut bundle = FluentBundle::new(vec![lang_id]);
        match bundle.add_resource(resource) {
            Ok(_) => {}
            Err(errors) => {
                return Err(crate::DejavuError::ParseError(format!("Failed to add resource to bundle: {:?}", errors)));
            }
        };

        self.bundles.insert(locale.to_string(), bundle);
        Ok(())
    }
}

impl TranslationProvider for FluentTranslator {
    fn translate(&self, key: &str, args: Option<&HashMap<String, String>>, locale: &str) -> DejavuResult<String> {
        // Fast path: try current locale first
        if let Some(bundle) = self.bundles.get(locale) {
            if let Some(message) = bundle.get_message(key) {
                if let Some(pattern) = message.value() {
                    let mut errors = vec![];
                    let args_map = args.map(|args| {
                        let mut args_map = fluent_bundle::FluentArgs::new();
                        for (name, value) in args {
                            args_map.set(name, value);
                        }
                        args_map
                    });
                    let result = bundle.format_pattern(pattern, args_map.as_ref(), &mut errors);
                    return Ok(result.to_string());
                }
            }
        }

        // Fallback: try default locale
        if locale != self.default_locale {
            if let Some(bundle) = self.bundles.get(&self.default_locale) {
                if let Some(message) = bundle.get_message(key) {
                    if let Some(pattern) = message.value() {
                        let mut errors = vec![];
                        let args_map = args.map(|args| {
                            let mut args_map = fluent_bundle::FluentArgs::new();
                            for (name, value) in args {
                                args_map.set(name, value);
                            }
                            args_map
                        });
                        let result = bundle.format_pattern(pattern, args_map.as_ref(), &mut errors);
                        return Ok(result.to_string());
                    }
                }
            }
        }

        // Return key if translation not found
        Ok(key.to_string())
    }

    fn get_locale(&self) -> &str {
        &self.current_locale
    }

    fn set_locale(&mut self, locale: &str) {
        self.current_locale = locale.to_string();
    }

    fn clone_box(&self) -> Box<dyn TranslationProvider> {
        // Create a new FluentTranslator with the same locales but empty bundles
        // This is a workaround because FluentBundle doesn't implement Clone
        let new_translator = FluentTranslator::new(&self.current_locale, &self.default_locale);
        Box::new(new_translator)
    }
}

impl FluentTranslator {
    /// Get a bundle for a locale, with fallback to default
    pub fn get_bundle(&self, locale: &str) -> Option<&FluentBundle<FluentResource>> {
        self.bundles
            .get(locale)
            .or_else(|| if locale != self.default_locale { self.bundles.get(&self.default_locale) } else { None })
    }
}
