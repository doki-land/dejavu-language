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
use toml_edit::Document;

/// Main DejaVu configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DejavuConfig {
    /// Template base directory
    pub template_dir: PathBuf,
}

impl DejavuConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self { template_dir: PathBuf::from("./templates") }
    }

    /// Load configuration from a TOML file
    pub fn load<P: Into<PathBuf>>(path: P) -> DejavuResult<Self> {
        let path = path.into();
        let config =
            Figment::new().merge(Toml::file(&path)).extract::<Self>().map_err(|e| DejavuError::InvalidConfig(e.to_string()))?;
        Ok(config)
    }

    /// Load configuration from the default location
    pub fn load_default() -> DejavuResult<Self> {
        let figment = Figment::new()
            .merge(Toml::file("dejavu.toml"))
            .merge(Toml::file(".dejavu.toml"))
            .merge(Env::prefixed("DEJAVU_").split("."));

        let config = figment.extract::<Self>().unwrap_or_else(|_| Self::new());
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save<P: Into<PathBuf>>(&self, path: P) -> DejavuResult<()> {
        let path = path.into();
        let content = toml::to_string_pretty(self).map_err(|e| DejavuError::InvalidConfig(e.to_string()))?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Edit configuration using toml_edit
    pub fn edit<P: Into<PathBuf>>(path: P, edit_fn: impl Fn(&mut toml_edit::Document<String>)) -> DejavuResult<()> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)?;
        let mut doc = content.parse::<toml_edit::Document<String>>().map_err(|e| DejavuError::InvalidConfig(e.to_string()))?;
        edit_fn(&mut doc);
        std::fs::write(&path, doc.to_string())?;
        Ok(())
    }
}
