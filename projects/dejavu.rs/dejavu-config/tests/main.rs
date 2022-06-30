use dejavu_config::DejavuConfig;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_new_config() {
    let config = DejavuConfig::new();
    assert_eq!(config.template_dir, PathBuf::from("./templates"));
}

#[test]
fn test_load_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("dejavu.toml");

    // Create a test config file
    std::fs::write(&config_path, "template_dir = \"test_templates\"").unwrap();

    // Load the config
    let config = DejavuConfig::load(&config_path).unwrap();
    assert_eq!(config.template_dir, PathBuf::from("test_templates"));
}

#[test]
fn test_load_default_config() {
    let temp_dir = tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();

    // Change to temp directory
    std::env::set_current_dir(temp_dir.path()).unwrap();

    // Create dejavu.toml in temp directory
    std::fs::write("dejavu.toml", "template_dir = \"default_templates\"").unwrap();

    // Load default config
    let config = DejavuConfig::load_default().unwrap();
    assert_eq!(config.template_dir, PathBuf::from("default_templates"));

    // Change back to original directory
    std::env::set_current_dir(original_dir).unwrap();
}

#[test]
fn test_save_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("dejavu.toml");

    // Create a config and save it
    let mut config = DejavuConfig::new();
    config.template_dir = PathBuf::from("saved_templates");
    config.save(&config_path).unwrap();

    // Load it back and verify
    let loaded_config = DejavuConfig::load(&config_path).unwrap();
    assert_eq!(loaded_config.template_dir, PathBuf::from("saved_templates"));
}
