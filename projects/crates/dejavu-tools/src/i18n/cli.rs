//! Command line interface for DejaVu Engine internationalization tools

use clap::{Arg, Command};
use std::path::Path;

use super::{I18nConfig, extractor::Extractor, validator::Validator};

/// Run the i18n CLI
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("dejavu-i18n")
        .version("1.0")
        .about("Internationalization tools for DejaVu Engine")
        .subcommand(
            Command::new("extract")
                .about("Extract translatable strings from source files")
                .arg(
                    Arg::new("source-dir")
                        .short('s')
                        .long("source-dir")
                        .help("Source directory to search for translatable content")
                        .default_value(".")
                        .num_args(1..),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output file for extracted translations")
                        .default_value("translations.pot"),
                )
                .arg(
                    Arg::new("translations-dir")
                        .short('t')
                        .long("translations-dir")
                        .help("Directory to store translation files")
                        .default_value("translations"),
                ),
        )
        .subcommand(
            Command::new("validate")
                .about("Validate translation files")
                .arg(
                    Arg::new("source-dir")
                        .short('s')
                        .long("source-dir")
                        .help("Source directory to search for translatable content")
                        .default_value(".")
                        .num_args(1..),
                )
                .arg(
                    Arg::new("translations-dir")
                        .short('t')
                        .long("translations-dir")
                        .help("Directory containing translation files")
                        .default_value("translations"),
                )
                .arg(
                    Arg::new("locales")
                        .short('l')
                        .long("locales")
                        .help("Supported locales (comma-separated)")
                        .default_value("en,zh-CN"),
                )
                .arg(Arg::new("default-locale").short('d').long("default-locale").help("Default locale").default_value("en"))
                .arg(
                    Arg::new("report")
                        .short('r')
                        .long("report")
                        .help("Generate validation report")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("extract", sub_matches)) => {
            let source_dirs: Vec<&str> = sub_matches.get_many::<String>("source-dir").unwrap().map(|s| s.as_str()).collect();
            let output = sub_matches.get_one::<String>("output").unwrap().as_str();
            let translations_dir = sub_matches.get_one::<String>("translations-dir").unwrap().as_str();

            // Create config
            let mut config = I18nConfig::new();
            for dir in source_dirs {
                config.add_source_dir(dir);
            }
            config.set_translations_dir(translations_dir);

            // Create extractor and extract translations
            let mut extractor = Extractor::new(config);
            let extracted = extractor.extract()?;

            // Save extracted translations
            extractor.save_extracted(output)?;

            println!("Extracted {} translations to {}", extracted.len(), output);
        }
        Some(("validate", sub_matches)) => {
            let source_dirs: Vec<&str> = sub_matches.get_many::<String>("source-dir").unwrap().map(|s| s.as_str()).collect();
            let translations_dir = sub_matches.get_one::<String>("translations-dir").unwrap().as_str();
            let locales_str = sub_matches.get_one::<String>("locales").unwrap().as_str();
            let default_locale = sub_matches.get_one::<String>("default-locale").unwrap().as_str();
            let generate_report = sub_matches.get_flag("report");

            // Parse locales
            let locales: Vec<String> = locales_str.split(',').map(|s| s.trim().to_string()).collect();

            // Create config
            let mut config = I18nConfig::new();
            for dir in source_dirs {
                config.add_source_dir(dir);
            }
            config.set_translations_dir(translations_dir);
            config.set_default_locale(default_locale);
            for locale in &locales {
                config.add_supported_locale(locale);
            }

            // Extract translations first
            let mut extractor = Extractor::new(config.clone());
            let extracted = extractor.extract()?;

            // Create validator and load translations
            let mut validator = Validator::new(config, extracted);
            validator.load_translations()?;

            // Validate translations
            let result = validator.validate();

            // Print validation results
            println!("Validation result: {}", if result.passed { "PASSED" } else { "FAILED" });
            println!("Total keys: {}", result.stats.total_keys);
            println!("Missing translations: {}", result.stats.missing_translations);
            println!("Unused translations: {}", result.stats.unused_translations);
            println!("Invalid translations: {}", result.stats.invalid_translations);

            // Print errors and warnings
            if !result.errors.is_empty() {
                println!("\nErrors:");
                for error in &result.errors {
                    println!("- {}", error.message);
                }
            }

            if !result.warnings.is_empty() {
                println!("\nWarnings:");
                for warning in &result.warnings {
                    println!("- {}", warning.message);
                }
            }

            // Generate report if requested
            if generate_report {
                let report = validator.generate_report(&result);
                let report_path = "validation-report.md";
                std::fs::write(report_path, report)?;
                println!("\nValidation report generated at: {}", report_path);
            }
        }
        _ => {
            println!("Use 'dejavu-i18n --help' for usage information");
        }
    }

    Ok(())
}
