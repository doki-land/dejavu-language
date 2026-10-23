//! Codegen input options parsed from JSON.
//!
//! Keep JSON shape permissive (unknown fields ignored) so future dejavu-engine
//! versions can pass extra knobs without breaking older crates.

use serde::{Deserialize, Serialize};

/// Single-template codegen options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodegenOptions {
    /// Emit runtime `throw` when an identifier resolves to `undefined`
    /// (matches dejavu-engine strictUndefined render behaviour).
    #[serde(default)]
    pub strict_undefined: bool,

    /// Template id / entry id for dependency tracking. If absent we fall back
    /// to `"__entry__"` (single template has no catalog).
    #[serde(default)]
    pub entry_id: Option<String>,
}

impl CodegenOptions {
    /// Parse from a JSON options string. Accepts `None` or missing → defaults.
    pub fn from_json(opts: Option<&str>) -> Result<Self, serde_json::Error> {
        let Some(json) = opts else {
            return Ok(Self::default());
        };
        if json.trim().is_empty() {
            return Ok(Self::default());
        }
        serde_json::from_str::<CodegenOptions>(json)
    }

    pub fn entry_id(&self) -> &str {
        self.entry_id.as_deref().unwrap_or("__entry__")
    }
}
