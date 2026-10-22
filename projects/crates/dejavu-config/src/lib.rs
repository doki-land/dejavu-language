#![warn(missing_docs)]
//! DejaVu Configuration
//!
//! This crate provides configuration management for the DejaVu template engine.
//! It uses figment for configuration loading and toml_edit for TOML manipulation.

use dejavu_types::{DejavuError, DejavuResult};
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default `template_dir` value (used by serde when all figment sources omit it).
fn default_template_dir() -> PathBuf {
    PathBuf::from("./templates")
}

/// Main DejaVu configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DejavuConfig {
    /// Template base directory
    #[serde(default = "default_template_dir")]
    pub template_dir: PathBuf,
}

impl Default for DejavuConfig {
    fn default() -> Self {
        Self { template_dir: default_template_dir() }
    }
}

impl DejavuConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from a TOML file
    pub fn load<P: Into<PathBuf>>(path: P) -> DejavuResult<Self> {
        let path = path.into();
        let config = Figment::new()
            .merge(Toml::file(&path))
            .extract::<Self>()
            .map_err(|e| DejavuError::InvalidConfig(e.to_string()))?;
        Ok(config)
    }

    /// Load configuration from the default location.
    ///
    /// Uses `dejavu.toml` / `.dejavu.toml` (cwd) plus env vars prefixed with `DEJAVU_`.
    /// All sources absent → serde default kicks in (no error, returns `Self::default()`).
    /// Any source present but invalid (malformed TOML, wrong env var type, …)
    /// → `DejavuError::InvalidConfig` propagated — no longer silently swallowed.
    pub fn load_default() -> DejavuResult<Self> {
        let config = Figment::new()
            .merge(Toml::file("dejavu.toml"))
            .merge(Toml::file(".dejavu.toml"))
            .merge(Env::prefixed("DEJAVU_").split("."))
            .extract::<Self>()
            .map_err(|e| DejavuError::InvalidConfig(e.to_string()))?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save<P: Into<PathBuf>>(&self, path: P) -> DejavuResult<()> {
        let path = path.into();
        let content =
            toml::to_string_pretty(self).map_err(|e| DejavuError::InvalidConfig(format!("failed to serialize config: {e}")))?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Edit configuration using toml_edit
    pub fn edit<P: Into<PathBuf>>(path: P, edit_fn: impl Fn(&mut toml_edit::Document<String>)) -> DejavuResult<()> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)?;
        let mut doc = content.parse::<toml_edit::Document<String>>().map_err(|e| {
            DejavuError::InvalidConfig(format!("failed to parse TOML document: {e}"))
        })?;
        edit_fn(&mut doc);
        std::fs::write(&path, doc.to_string())?;
        Ok(())
    }
}
