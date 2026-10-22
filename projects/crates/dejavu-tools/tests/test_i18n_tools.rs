//! Tests for DejaVu Engine internationalization tools
//!
//! 每个测试用独立临时目录隔离，避免扫到整个 workspace。

use std::{
    env,
    fs,
    path::{Path, PathBuf},
};

use dejavu_tools::i18n::{I18nConfig, extractor::Extractor, validator::Validator};

/// 为每个测试创建独立临时子目录，返回该路径。
fn unique_temp_dir(name: &str) -> PathBuf {
    let mut p = env::temp_dir();
    let pid = std::process::id();
    let rand = rand_suffix();
    p.push(format!("dejavu-tools-{name}-{pid}-{rand}"));
    fs::create_dir_all(&p).unwrap();
    p
}

fn rand_suffix() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u32
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

#[test]
fn test_extractor() {
    let dir = unique_temp_dir("extractor");
    let template = dir.join("test_template.dejavu");
    fs::write(
        &template,
        r#"Hello, <% name %>!
<% t("greeting") %>
<% t('welcome') %>"#,
    )
    .unwrap();

    let mut config = I18nConfig::new();
    config.add_source_dir(dir.to_str().unwrap());
    config.add_include_pattern("*.dejavu");

    let mut extractor = Extractor::new(config);
    let extracted = extractor.extract().unwrap();

    assert!(extracted.contains_key("greeting"));
    assert!(extracted.contains_key("welcome"));
    assert_eq!(extracted.len(), 2);

    cleanup(&dir);
}

#[test]
fn test_validator() {
    let dir = unique_temp_dir("validator-ok");

    // 创建模板
    let template = dir.join("test_template.dejavu");
    fs::write(
        &template,
        r#"Hello, <% name %>!
<% t("greeting") %>"#,
    )
    .unwrap();

    // 创建翻译目录和文件
    let translations = dir.join("translations");
    fs::create_dir_all(&translations).unwrap();
    fs::write(
        translations.join("en.po"),
        r#"msgid "greeting"
msgstr "Hello"
"#,
    )
    .unwrap();
    fs::write(
        translations.join("zh-CN.po"),
        r#"msgid "greeting"
msgstr "你好"
"#,
    )
    .unwrap();

    let mut config = I18nConfig::new();
    config.add_source_dir(dir.to_str().unwrap());
    config.set_translations_dir(translations.to_str().unwrap());
    config.set_default_locale("en");
    config.add_supported_locale("en");
    config.add_supported_locale("zh-CN");

    let mut extractor = Extractor::new(config.clone());
    let extracted = extractor.extract().unwrap();

    let mut validator = Validator::new(config, extracted);
    validator.load_translations().unwrap();
    let result = validator.validate();

    assert!(result.passed, "validation errors: {:?}", result.errors);
    assert_eq!(result.errors.len(), 0);

    cleanup(&dir);
}

#[test]
fn test_extractor_with_rust_file() {
    let dir = unique_temp_dir("extractor-rust");
    let rust_file = dir.join("test_rust.rs");
    fs::write(
        &rust_file,
        r#"fn main() {
    println!("{}", t("hello"));
    println!("{}", t("world"));
}"#,
    )
    .unwrap();

    let mut config = I18nConfig::new();
    config.add_source_dir(dir.to_str().unwrap());
    config.add_include_pattern("*.rs");

    let mut extractor = Extractor::new(config);
    let extracted = extractor.extract().unwrap();

    assert!(extracted.contains_key("hello"));
    assert!(extracted.contains_key("world"));
    assert_eq!(extracted.len(), 2);

    cleanup(&dir);
}

#[test]
fn test_validator_missing_translation() {
    let dir = unique_temp_dir("validator-missing");

    // 创建模板
    let template = dir.join("test_template.dejavu");
    fs::write(
        &template,
        r#"Hello, <% name %>!
<% t("greeting") %>
<% t("missing") %>"#,
    )
    .unwrap();

    // 创建翻译目录和文件（只有 greeting，缺 missing）
    let translations = dir.join("translations");
    fs::create_dir_all(&translations).unwrap();
    fs::write(
        translations.join("en.po"),
        r#"msgid "greeting"
msgstr "Hello"
"#,
    )
    .unwrap();

    let mut config = I18nConfig::new();
    config.add_source_dir(dir.to_str().unwrap());
    config.set_translations_dir(translations.to_str().unwrap());
    config.set_default_locale("en");
    config.add_supported_locale("en");

    let mut extractor = Extractor::new(config.clone());
    let extracted = extractor.extract().unwrap();

    let mut validator = Validator::new(config, extracted);
    validator.load_translations().unwrap();
    let result = validator.validate();

    assert!(!result.passed, "should have failed due to missing translation");

    // en.po 存在但缺 "missing" 这个 key，所以应该至少有 1 个 MissingTranslationKey 错误
    let missing_key_errors: Vec<_> = result
        .errors
        .iter()
        .filter(|e| matches!(e.error_type, dejavu_tools::i18n::validator::ValidationErrorType::MissingTranslationKey))
        .collect();
    assert!(
        !missing_key_errors.is_empty(),
        "expected MissingTranslationKey errors, got: {:?}",
        result.errors
    );

    cleanup(&dir);
}
