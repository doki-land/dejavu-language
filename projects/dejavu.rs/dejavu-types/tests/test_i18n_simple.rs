#![cfg(test)]
use dejavu_types::i18n::*;
use std::{collections::HashMap, time::SystemTime};

#[test]
fn test_translation_provider_trait() {
    // Test SimpleTranslator implements TranslationProvider
    let mut translator = SimpleTranslator::new("en", "en");

    let mut en_translations = HashMap::new();
    en_translations.insert("greeting".to_string(), "Hello!".to_string());
    translator.add_translations("en", en_translations);

    // Test get_locale
    assert_eq!(translator.get_locale(), "en");

    // Test set_locale
    translator.set_locale("fr");
    assert_eq!(translator.get_locale(), "fr");

    // Test translate
    let result = translator.translate("greeting", None, "en").unwrap();
    assert_eq!(result, "Hello!");

    // Test clone_box
    let cloned_translator = translator.clone_box();
    assert_eq!(cloned_translator.get_locale(), "fr");
}

#[test]
fn test_translation_data() {
    let mut data = TranslationData::new("en", "en");

    // Test add_translations
    let mut en_translations = HashMap::new();
    en_translations.insert("greeting".to_string(), "Hello!".to_string());
    data.add_translations("en", en_translations);

    let mut fr_translations = HashMap::new();
    fr_translations.insert("greeting".to_string(), "Bonjour!".to_string());
    data.add_translations("fr", fr_translations);

    // Test get_translation
    assert_eq!(data.get_translation("greeting", "en"), Some(&"Hello!".to_string()));
    assert_eq!(data.get_translation("greeting", "fr"), Some(&"Bonjour!".to_string()));

    // Test fallback to default locale
    assert_eq!(data.get_translation("greeting", "es"), Some(&"Hello!".to_string()));

    // Test missing key
    assert_eq!(data.get_translation("missing", "en"), None);
}

#[test]
fn test_simple_translator() {
    let mut translator = SimpleTranslator::new("en", "en");

    let mut en_translations = HashMap::new();
    en_translations.insert("greeting".to_string(), "Hello, {name}!".to_string());
    en_translations.insert("welcome".to_string(), "Welcome to DejaVu!".to_string());
    translator.add_translations("en", en_translations);

    let mut zh_translations = HashMap::new();
    zh_translations.insert("greeting".to_string(), "你好，{name}！".to_string());
    zh_translations.insert("welcome".to_string(), "欢迎使用 DejaVu！".to_string());
    translator.add_translations("zh", zh_translations);

    // Test English translations
    assert_eq!(translator.get_locale(), "en");

    let mut args = HashMap::new();
    args.insert("name".to_string(), "Alice".to_string());

    let result = translator.translate("greeting", Some(&args), "en").unwrap();
    assert_eq!(result, "Hello, Alice!");

    let result = translator.translate("welcome", None, "en").unwrap();
    assert_eq!(result, "Welcome to DejaVu!");

    // Test Chinese translations
    translator.set_locale("zh");
    assert_eq!(translator.get_locale(), "zh");

    let result = translator.translate("greeting", Some(&args), "zh").unwrap();
    assert_eq!(result, "你好，Alice！");

    let result = translator.translate("welcome", None, "zh").unwrap();
    assert_eq!(result, "欢迎使用 DejaVu！");

    // Test fallback to default locale
    let result = translator.translate("missing_key", None, "zh").unwrap();
    assert_eq!(result, "missing_key");
}

#[test]
fn test_i18n_context() {
    // Create translator with initial translations
    let mut simple_translator = SimpleTranslator::new("en", "en");

    let mut en_translations = HashMap::new();
    en_translations.insert("greeting".to_string(), "Hello, {name}!".to_string());
    en_translations.insert("welcome".to_string(), "Welcome to DejaVu!".to_string());
    simple_translator.add_translations("en", en_translations);

    let mut zh_translations = HashMap::new();
    zh_translations.insert("greeting".to_string(), "你好，{name}！".to_string());
    zh_translations.insert("welcome".to_string(), "欢迎使用 DejaVu！".to_string());
    simple_translator.add_translations("zh", zh_translations);

    let translator = Box::new(simple_translator);
    let mut i18n = I18nContext::new(translator);

    // Test translation
    assert_eq!(i18n.get_locale(), "en");

    let mut args = HashMap::new();
    args.insert("name".to_string(), "Alice".to_string());

    let result = i18n.t("greeting", Some(&args)).unwrap();
    assert_eq!(result, "Hello, Alice!");

    // Test cache - second call should use cache
    let result = i18n.t("greeting", Some(&args)).unwrap();
    assert_eq!(result, "Hello, Alice!");

    // Test language switching
    i18n.set_locale("zh");
    assert_eq!(i18n.get_locale(), "zh");

    let result = i18n.t("greeting", Some(&args)).unwrap();
    assert_eq!(result, "你好，Alice！");

    // Test number formatting
    let number_result = i18n.format_number(1234.56);
    assert_eq!(number_result, "1234。56");

    i18n.set_locale("en");
    let number_result = i18n.format_number(1234.56);
    assert_eq!(number_result, "1234.56");

    // Test date formatting
    let now = SystemTime::now();
    let date_result = i18n.format_date(now);
    assert_eq!(date_result, "Today");

    i18n.set_locale("zh");
    let date_result = i18n.format_date(now);
    assert_eq!(date_result, "今天");
}

#[test]
fn test_i18n_context_with_custom_cache() {
    // Create translator with initial translations
    let mut simple_translator = SimpleTranslator::new("en", "en");

    let mut en_translations = HashMap::new();
    en_translations.insert("test1".to_string(), "Test 1".to_string());
    en_translations.insert("test2".to_string(), "Test 2".to_string());
    simple_translator.add_translations("en", en_translations);

    let translator = Box::new(simple_translator);
    let mut i18n = I18nContext::with_cache_capacity(translator, 50);

    // Test translations
    let result1 = i18n.t("test1", None).unwrap();
    assert_eq!(result1, "Test 1");

    let result2 = i18n.t("test2", None).unwrap();
    assert_eq!(result2, "Test 2");

    // Test cache size
    assert_eq!(i18n.cache_size(), 2);

    // Test clear cache
    i18n.clear_cache();
    assert_eq!(i18n.cache_size(), 0);
}

#[test]
fn test_i18n_context_batch_translation() {
    // Create translator with initial translations
    let mut simple_translator = SimpleTranslator::new("en", "en");

    let mut en_translations = HashMap::new();
    en_translations.insert("greeting".to_string(), "Hello!".to_string());
    en_translations.insert("welcome".to_string(), "Welcome!".to_string());
    en_translations.insert("goodbye".to_string(), "Goodbye!".to_string());
    simple_translator.add_translations("en", en_translations);

    let translator = Box::new(simple_translator);
    let mut i18n = I18nContext::new(translator);

    // Test batch translation
    let keys = vec![("greeting".to_string(), None), ("welcome".to_string(), None), ("goodbye".to_string(), None)];

    let results = i18n.t_batch(&keys).unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0], "Hello!");
    assert_eq!(results[1], "Welcome!");
    assert_eq!(results[2], "Goodbye!");
}

#[test]
fn test_fluent_translator() {
    use std::path::Path;

    // Create a temporary FTL file for testing
    let ftl_content = r#"
greeting = Hello, { $name }!
welcome = Welcome to DejaVu!
"#;
    let ftl_path = Path::new("test_en.ftl");
    std::fs::write(ftl_path, ftl_content).unwrap();

    // Create FluentTranslator
    let mut translator = FluentTranslator::new("en", "en");
    translator.add_translations_from_file("en", ftl_path).unwrap();

    // Test translation
    let mut args = HashMap::new();
    args.insert("name".to_string(), "Alice".to_string());

    let result = translator.translate("greeting", Some(&args), "en").unwrap();
    assert!(result.contains("Hello"));
    assert!(result.contains("Alice"));

    let result = translator.translate("welcome", None, "en").unwrap();
    assert_eq!(result, "Welcome to DejaVu!");

    // Test fallback to default locale
    let result = translator.translate("greeting", Some(&args), "fr").unwrap();
    assert!(result.contains("Hello"));
    assert!(result.contains("Alice"));

    // Test missing key
    let result = translator.translate("missing", None, "en").unwrap();
    assert_eq!(result, "missing");

    // Clean up
    std::fs::remove_file(ftl_path).unwrap();
}

#[test]
fn test_fluent_translator_from_string() {
    // Create FluentTranslator from string
    let mut translator = FluentTranslator::new("en", "en");

    let ftl_content = r#"
greeting = Hello, { $name }!
welcome = Welcome to DejaVu!
"#;
    translator.add_translations_from_string("en", ftl_content).unwrap();

    // Test translation
    let mut args = HashMap::new();
    args.insert("name".to_string(), "Alice".to_string());

    let result = translator.translate("greeting", Some(&args), "en").unwrap();
    assert!(result.contains("Hello"));
    assert!(result.contains("Alice"));

    let result = translator.translate("welcome", None, "en").unwrap();
    assert_eq!(result, "Welcome to DejaVu!");
}
