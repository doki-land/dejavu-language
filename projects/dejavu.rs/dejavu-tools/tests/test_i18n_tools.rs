//! Tests for DejaVu Engine internationalization tools

use std::{fs, path::Path};

use dejavu_tools::i18n::{I18nConfig, extractor::Extractor, validator::Validator};

#[test]
fn test_extractor() {
    // Create a temporary test file
    let test_content = r#"Hello {{ name }}
{{ t("greeting") }}
{{ t('welcome') }}"#;
    let test_file = "test_template.dejavu";
    fs::write(test_file, test_content).unwrap();

    // Create config
    let mut config = I18nConfig::new();
    config.add_source_dir(".");
    config.add_include_pattern("*.dejavu");

    // Create extractor and extract translations
    let mut extractor = Extractor::new(config);
    let extracted = extractor.extract().unwrap();

    // Check that translations were extracted
    assert!(extracted.contains_key("greeting"));
    assert!(extracted.contains_key("welcome"));
    assert_eq!(extracted.len(), 2);

    // Clean up
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_validator() {
    // Create temporary test files
    let test_content = r#"Hello {{ name }}
{{ t("greeting") }}"#;
    let test_file = "test_template.dejavu";
    fs::write(test_file, test_content).unwrap();

    // Create translations directory and files
    fs::create_dir_all("translations").unwrap();
    let en_content = r#"msgid "greeting"
msgstr "Hello"
"#;
    let zh_content = r#"msgid "greeting"
msgstr "你好"
"#;
    fs::write("translations/en.po", en_content).unwrap();
    fs::write("translations/zh-CN.po", zh_content).unwrap();

    // Create config
    let mut config = I18nConfig::new();
    config.add_source_dir(".");
    config.set_translations_dir("translations");
    config.set_default_locale("en");
    config.add_supported_locale("en");
    config.add_supported_locale("zh-CN");

    // Extract translations
    let mut extractor = Extractor::new(config.clone());
    let extracted = extractor.extract().unwrap();

    // Create validator and validate
    let mut validator = Validator::new(config, extracted);
    validator.load_translations().unwrap();
    let result = validator.validate();

    // Check validation result
    assert!(result.passed);
    assert_eq!(result.errors.len(), 0);

    // Clean up
    fs::remove_file(test_file).unwrap();
    fs::remove_file("translations/en.po").unwrap();
    fs::remove_file("translations/zh-CN.po").unwrap();
    fs::remove_dir("translations").unwrap();
}

#[test]
fn test_extractor_with_rust_file() {
    // Create a temporary Rust file
    let test_content = r#"fn main() {
    println!("{}", t("hello"));
    println!("{}", t("world"));
}"#;
    let test_file = "test_rust.rs";
    fs::write(test_file, test_content).unwrap();

    // Create config
    let mut config = I18nConfig::new();
    config.add_source_dir(".");
    config.add_include_pattern("*.rs");

    // Create extractor and extract translations
    let mut extractor = Extractor::new(config);
    let extracted = extractor.extract().unwrap();

    // Check that translations were extracted
    assert!(extracted.contains_key("hello"));
    assert!(extracted.contains_key("world"));
    assert_eq!(extracted.len(), 2);

    // Clean up
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_validator_missing_translation() {
    // Create temporary test files
    let test_content = r#"Hello {{ name }}
{{ t("greeting") }}
{{ t("missing") }}"#;
    let test_file = "test_template.dejavu";
    fs::write(test_file, test_content).unwrap();

    // Create translations directory and files
    fs::create_dir_all("translations").unwrap();
    let en_content = r#"msgid "greeting"
msgstr "Hello"
"#;
    fs::write("translations/en.po", en_content).unwrap();

    // Create config
    let mut config = I18nConfig::new();
    config.add_source_dir(".");
    config.set_translations_dir("translations");
    config.set_default_locale("en");
    config.add_supported_locale("en");

    // Extract translations
    let mut extractor = Extractor::new(config.clone());
    let extracted = extractor.extract().unwrap();

    // Create validator and validate
    let mut validator = Validator::new(config, extracted);
    validator.load_translations().unwrap();
    let result = validator.validate();

    // Check validation result
    assert!(!result.passed);
    assert_eq!(result.errors.len(), 1);

    // Clean up
    fs::remove_file(test_file).unwrap();
    fs::remove_file("translations/en.po").unwrap();
    fs::remove_dir("translations").unwrap();
}
