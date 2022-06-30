//! Multilingual support tests for internationalization functionality

use dejavu_types::i18n::*;
use std::{collections::HashMap, time::SystemTime};

#[test]
fn test_multilingual_translations() {
    // Create translator with multiple languages
    let mut translator = SimpleTranslator::new("en", "en");

    // English translations
    let mut en_translations = HashMap::new();
    en_translations.insert("greeting".to_string(), "Hello, {name}!".to_string());
    en_translations.insert("welcome".to_string(), "Welcome to DejaVu!".to_string());
    en_translations.insert("farewell".to_string(), "Goodbye!".to_string());
    translator.add_translations("en", en_translations);

    // Chinese translations
    let mut zh_translations = HashMap::new();
    zh_translations.insert("greeting".to_string(), "你好，{name}！".to_string());
    zh_translations.insert("welcome".to_string(), "欢迎使用 DejaVu！".to_string());
    zh_translations.insert("farewell".to_string(), "再见！".to_string());
    translator.add_translations("zh", zh_translations);

    // French translations
    let mut fr_translations = HashMap::new();
    fr_translations.insert("greeting".to_string(), "Bonjour, {name}!".to_string());
    fr_translations.insert("welcome".to_string(), "Bienvenue sur DejaVu!".to_string());
    translator.add_translations("fr", fr_translations);

    // Test English
    translator.set_locale("en");
    let mut args = HashMap::new();
    args.insert("name".to_string(), "Alice".to_string());

    let result = translator.translate("greeting", Some(&args), "en").unwrap();
    assert_eq!(result, "Hello, Alice!");

    let result = translator.translate("welcome", None, "en").unwrap();
    assert_eq!(result, "Welcome to DejaVu!");

    // Test Chinese
    translator.set_locale("zh");
    let result = translator.translate("greeting", Some(&args), "zh").unwrap();
    assert_eq!(result, "你好，Alice！");

    let result = translator.translate("welcome", None, "zh").unwrap();
    assert_eq!(result, "欢迎使用 DejaVu！");

    // Test French (missing farewell translation)
    translator.set_locale("fr");
    let result = translator.translate("greeting", Some(&args), "fr").unwrap();
    assert_eq!(result, "Bonjour, Alice!");

    let result = translator.translate("welcome", None, "fr").unwrap();
    assert_eq!(result, "Bienvenue sur DejaVu!");

    // Test fallback for missing French translation
    let result = translator.translate("farewell", None, "fr").unwrap();
    assert_eq!(result, "Goodbye!"); // Should fallback to English
}

#[test]
fn test_multilingual_formatting() {
    // Create I18n context
    let translator = Box::new(SimpleTranslator::new("en", "en"));
    let mut i18n = I18nContext::new(translator);

    // Test number formatting
    i18n.set_locale("en");
    let en_number = i18n.format_number(1234.56);
    assert_eq!(en_number, "1234.56");

    i18n.set_locale("zh");
    let zh_number = i18n.format_number(1234.56);
    assert_eq!(zh_number, "1234。56");

    // Test date formatting
    let now = SystemTime::now();

    i18n.set_locale("en");
    let en_date = i18n.format_date(now);
    assert_eq!(en_date, "Today");

    i18n.set_locale("zh");
    let zh_date = i18n.format_date(now);
    assert_eq!(zh_date, "今天");
}

#[test]
fn test_locale_fallback_chain() {
    // Create translator with fallback chain
    let mut translator = SimpleTranslator::new("zh-CN", "en");

    // English (default) translations
    let mut en_translations = HashMap::new();
    en_translations.insert("greeting".to_string(), "Hello!".to_string());
    en_translations.insert("welcome".to_string(), "Welcome!".to_string());
    en_translations.insert("farewell".to_string(), "Goodbye!".to_string());
    translator.add_translations("en", en_translations);

    // Chinese (Simplified) translations
    let mut zh_cn_translations = HashMap::new();
    zh_cn_translations.insert("greeting".to_string(), "你好！".to_string());
    zh_cn_translations.insert("welcome".to_string(), "欢迎！".to_string());
    translator.add_translations("zh-CN", zh_cn_translations);

    // Test exact locale match
    let result = translator.translate("greeting", None, "zh-CN").unwrap();
    assert_eq!(result, "你好！");

    // Test fallback to default locale for missing translation
    let result = translator.translate("farewell", None, "zh-CN").unwrap();
    assert_eq!(result, "Goodbye!");

    // Test fallback to default locale for unsupported locale
    let result = translator.translate("greeting", None, "fr").unwrap();
    assert_eq!(result, "Hello!");
}
