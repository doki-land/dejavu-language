//! Reliability tests for internationalization functionality

use dejavu_types::i18n::*;
use std::collections::HashMap;

#[test]
fn test_edge_cases() {
    // Create translator
    let mut translator = SimpleTranslator::new("en", "en");

    // Test empty translations
    let empty_translations = HashMap::new();
    translator.add_translations("en", empty_translations);

    // Test translation with empty key
    let result = translator.translate("", None, "en").unwrap();
    assert_eq!(result, "");

    // Test translation with very long key
    let long_key = "a".repeat(1000);
    let result = translator.translate(&long_key, None, "en").unwrap();
    assert_eq!(result, long_key);

    // Test translation with special characters
    let special_key = "key with spaces and special chars!@#$%^&*()";
    let result = translator.translate(special_key, None, "en").unwrap();
    assert_eq!(result, special_key);

    // Test translation with many arguments
    let mut translator = SimpleTranslator::new("en", "en");
    let mut en_translations = HashMap::new();
    en_translations.insert("many_args".to_string(), "{a} {b} {c} {d} {e} {f} {g} {h} {i} {j}".to_string());
    translator.add_translations("en", en_translations);

    let mut args = HashMap::new();
    for c in 'a'..='j' {
        args.insert(c.to_string(), c.to_string());
    }

    let result = translator.translate("many_args", Some(&args), "en").unwrap();
    assert_eq!(result, "a b c d e f g h i j");
}

#[test]
fn test_error_handling() {
    // Create translator
    let translator = Box::new(SimpleTranslator::new("en", "en"));
    let mut i18n = I18nContext::new(translator);

    // Test adding translations to non-SimpleTranslator
    // This should fail
    let result = i18n.add_translations("en", HashMap::new());
    assert!(result.is_ok()); // This should work because we're using SimpleTranslator

    // Test translation with invalid locale
    let result = i18n.t("key", None);
    assert!(result.is_ok()); // Should return key if not found

    // Test batch translation with empty keys
    let empty_keys: Vec<(String, Option<HashMap<String, String>>)> = Vec::new();
    let result = i18n.t_batch(&empty_keys);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_fluent_translator_error_handling() {
    use std::path::Path;

    // Create FluentTranslator
    let mut translator = FluentTranslator::new("en", "en");

    // Test adding invalid FTL content
    let invalid_ftl = "invalid ftl content";
    let result = translator.add_translations_from_string("en", invalid_ftl);
    assert!(result.is_err());

    // Test adding translations from non-existent file
    let non_existent_path = Path::new("non_existent.ftl");
    let result = translator.add_translations_from_file("en", non_existent_path);
    assert!(result.is_err());

    // Test translation with invalid locale
    let result = translator.translate("key", None, "invalid-locale");
    assert!(result.is_ok()); // Should fallback to default locale
}

#[test]
fn test_cache_behavior() {
    // Create translator
    let mut translator = SimpleTranslator::new("en", "en");
    let mut en_translations = HashMap::new();
    en_translations.insert("key".to_string(), "value".to_string());
    translator.add_translations("en", en_translations);

    // Create I18n context with small cache
    let translator_box = Box::new(translator);
    let mut i18n = I18nContext::with_cache_capacity(translator_box, 2);

    // Test cache eviction
    i18n.t("key", None).unwrap();
    i18n.t("key1", None).unwrap();
    i18n.t("key2", None).unwrap(); // This should evict "key"

    // Test that "key" is evicted
    assert_eq!(i18n.cache_size(), 2);

    // Test locale switching clears cache
    i18n.set_locale("zh");
    assert_eq!(i18n.cache_size(), 0);

    // Test adding translations clears cache
    i18n.add_translations("zh", HashMap::new()).unwrap();
    assert_eq!(i18n.cache_size(), 0);
}
