//! Codegen input options parsed from JSON.
//!
//! Keep JSON shape permissive (unknown fields ignored) so future dejavu-engine
//! versions can pass extra knobs without breaking older crates.

use dejavu_ir::registry::{CapabilitySet, CustomEntry};
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

    /// Registry capabilities the compiled artifact may use. `None` grants every
    /// capability; a list closes the rest at compile time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<String>>,

    /// Host-supplied `filter` / `function` / `test` descriptions merged after the
    /// built-in tables. Validated once per compile; implementations are supplied
    /// by the host at render time (`render(ctx, { custom })`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_registry: Vec<CustomEntry>,
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

    /// Resolve the capability allowlist, returning the first unknown name.
    pub fn capability_set(&self) -> Result<CapabilitySet, String> {
        match &self.capabilities {
            Some(names) => CapabilitySet::from_names(names),
            None => Ok(CapabilitySet::all()),
        }
    }
}
