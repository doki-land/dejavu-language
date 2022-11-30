//! Performance tests for internationalization functionality

use dejavu_types::i18n::*;
use std::{collections::HashMap, time::Instant};

#[test]
fn test_translation_performance() {
    // Create translator with multiple languages
    let mut translator = SimpleTranslator::new("en", "en");

    // Add translations for multiple languages
    let mut en_translations = HashMap::new();
    for i in 0..1000 {
        en_translations.insert(format!("key_{}", i), format!("English value {}", i));
    }
    translator.add_translations("en", en_translations);

    let mut zh_translations = HashMap::new();
    for i in 0..1000 {
        zh_translations.insert(format!("key_{}", i), format!("中文值 {}", i));
    }
    translator.add_translations("zh", zh_translations);

    // Create I18n context with cache
    let translator_box = Box::new(translator);
    let mut i18n = I18nContext::new(translator_box);

    // Test performance with cache
    let start = Instant::now();

    // First run (should populate cache)
    for i in 0..1000 {
        let key = format!("key_{}", i);
        i18n.t(&key, None).unwrap();
    }

    let first_run_time = start.elapsed();
    println!("First run time: {:?}", first_run_time);

    // Second run (should use cache)
    let start = Instant::now();

    for i in 0..1000 {
        let key = format!("key_{}", i);
        i18n.t(&key, None).unwrap();
    }

    let second_run_time = start.elapsed();
    println!("Second run time: {:?}", second_run_time);

    // Verify that cache is working (second run should be faster)
    assert!(second_run_time < first_run_time, "Cache is not working properly");

    // Test cache size
    assert_eq!(i18n.cache_size(), 1000);
}

#[test]
fn test_batch_translation_performance() {
    // Create translator
    let mut translator = SimpleTranslator::new("en", "en");

    // Add translations
    let mut en_translations = HashMap::new();
    for i in 0..1000 {
        en_translations.insert(format!("key_{}", i), format!("Value {}", i));
    }
    translator.add_translations("en", en_translations);

    // Test batch vs individual with cold cache
    let translator_box1 = Box::new(translator.clone());
    let mut i18n_batch = I18nContext::new(translator_box1);

    let translator_box2 = Box::new(translator);
    let mut i18n_individual = I18nContext::new(translator_box2);

    // Prepare batch translation keys
    let mut keys = Vec::with_capacity(1000);
    for i in 0..1000 {
        keys.push((format!("key_{}", i), None));
    }

    // Test batch translation performance (cold cache)
    let start = Instant::now();
    let results = i18n_batch.t_batch(&keys).unwrap();
    let batch_time = start.elapsed();

    println!("Batch translation time for 1000 keys: {:?}", batch_time);
    assert_eq!(results.len(), 1000);

    // Compare with individual translations (cold cache)
    let start = Instant::now();
    for i in 0..1000 {
        i18n_individual.t(&format!("key_{}", i), None).unwrap();
    }
    let individual_time = start.elapsed();

    println!("Individual translation time for 1000 keys: {:?}", individual_time);

    // Batch translation should be at least as fast as individual
    // Allow some tolerance for the overhead of batch processing
    assert!(batch_time < individual_time * 2, "Batch translation should not be significantly slower than individual");
}

#[test]
fn test_locale_switching_performance() {
    // Create translator with multiple languages
    let mut translator = SimpleTranslator::new("en", "en");

    // Add translations for multiple languages
    let mut en_translations = HashMap::new();
    for i in 0..1000 {
        en_translations.insert(format!("key_{}", i), format!("English {}", i));
    }
    translator.add_translations("en", en_translations);

    let mut zh_translations = HashMap::new();
    for i in 0..1000 {
        zh_translations.insert(format!("key_{}", i), format!("中文 {}", i));
    }
    translator.add_translations("zh", zh_translations);

    // Create I18n context
    let translator_box = Box::new(translator);
    let mut i18n = I18nContext::new(translator_box);

    // Test locale switching performance
    let start = Instant::now();

    for _ in 0..100 {
        // Switch to English
        i18n.set_locale("en");
        // Translate some keys
        for i in 0..10 {
            i18n.t(&format!("key_{}", i), None).unwrap();
        }

        // Switch to Chinese
        i18n.set_locale("zh");
        // Translate some keys
        for i in 0..10 {
            i18n.t(&format!("key_{}", i), None).unwrap();
        }
    }

    let locale_switch_time = start.elapsed();
    println!("Locale switching performance: {:?}", locale_switch_time);

    // Verify that translations are correct after switching
    i18n.set_locale("en");
    let en_result = i18n.t("key_0", None).unwrap();
    assert!(en_result.contains("English"));

    i18n.set_locale("zh");
    let zh_result = i18n.t("key_0", None).unwrap();
    assert!(zh_result.contains("中文"));
}
