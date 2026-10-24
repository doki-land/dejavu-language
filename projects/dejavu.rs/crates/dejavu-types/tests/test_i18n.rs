#![cfg(test)]
use dejavu_types::i18n::*;
use std::{collections::HashMap, time::SystemTime};

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
    let translator = Box::new(SimpleTranslator::new("en", "en"));
    let mut i18n = I18nContext::new(translator);

    // Add translations
    let mut en_translations = HashMap::new();
    en_translations.insert("greeting".to_string(), "Hello, {name}!".to_string());
    en_translations.insert("welcome".to_string(), "Welcome to DejaVu!".to_string());
    i18n.add_translations("en", en_translations).unwrap();

    let mut zh_translations = HashMap::new();
    zh_translations.insert("greeting".to_string(), "你好，{name}！".to_string());
    zh_translations.insert("welcome".to_string(), "欢迎使用 DejaVu！".to_string());
    i18n.add_translations("zh", zh_translations).unwrap();

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
    let translator = Box::new(SimpleTranslator::new("en", "en"));
    let mut i18n = I18nContext::with_cache_capacity(translator, 50);

    // Add translations
    let mut en_translations = HashMap::new();
    en_translations.insert("test1".to_string(), "Test 1".to_string());
    en_translations.insert("test2".to_string(), "Test 2".to_string());
    i18n.add_translations("en", en_translations).unwrap();

    // Test translations
    let result1 = i18n.t("test1", None).unwrap();
    assert_eq!(result1, "Test 1");

    let result2 = i18n.t("test2", None).unwrap();
    assert_eq!(result2, "Test 2");
}
