//! Language-neutral description of the built-in filters.
//!
//! Every host (Rust JIT, WASM, TypeScript AOT runtime) must agree on the same
//! names, aliases, arity and output capability. This module is the Rust source
//! of truth: the JIT renderer and the AOT compiler validate against it, and
//! `examples/filter_registry.rs` dumps the same table for cross-host checks.
//!
//! This is intentionally a static table, not a runtime registry: no host may
//! add filters that the other hosts do not implement.

/// Whether a filter's result must skip automatic HTML escaping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterCapability {
    /// Ordinary value transformation. The interpolation pipeline still escapes.
    Pure,
    /// The filter owns the output escaping (`escape`/`e`) or explicitly opts out
    /// of escaping (`safe`/`raw`). The pipeline must not escape again.
    SafeOutput,
}

/// One built-in filter contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilterSpec {
    /// Canonical name used by dispatch.
    pub name: &'static str,
    /// Accepted alternative spellings, all dispatching to `name`.
    pub aliases: &'static [&'static str],
    /// Minimum number of explicit arguments (`value` itself is not counted).
    pub min_args: usize,
    /// Maximum number of explicit arguments.
    pub max_args: usize,
    /// Output escaping capability.
    pub capability: FilterCapability,
}

const fn spec(
    name: &'static str,
    aliases: &'static [&'static str],
    min_args: usize,
    max_args: usize,
    capability: FilterCapability,
) -> FilterSpec {
    FilterSpec { name, aliases, min_args, max_args, capability }
}

/// The complete built-in filter set, in contract order.
pub const FILTERS: &[FilterSpec] = &[
    spec("uppercase", &["upper"], 0, 0, FilterCapability::Pure),
    spec("lowercase", &["lower"], 0, 0, FilterCapability::Pure),
    spec("trim", &[], 0, 0, FilterCapability::Pure),
    spec("default", &[], 1, 1, FilterCapability::Pure),
    spec("length", &[], 0, 0, FilterCapability::Pure),
    spec("join", &[], 0, 1, FilterCapability::Pure),
    spec("replace", &[], 2, 2, FilterCapability::Pure),
    spec("escape", &["e"], 0, 0, FilterCapability::SafeOutput),
    spec("safe", &["raw"], 0, 0, FilterCapability::SafeOutput),
    spec("date", &[], 0, 1, FilterCapability::Pure),
    spec("slug", &[], 0, 0, FilterCapability::Pure),
];

/// Look up a filter by canonical name or alias.
pub fn filter_spec(name: &str) -> Option<&'static FilterSpec> {
    FILTERS
        .iter()
        .find(|spec| spec.name == name || spec.aliases.contains(&name))
}

/// Validate a filter name and argument count.
///
/// Returns the canonical spec on success, or the index of the violated rule:
/// `FilterLookupError::Unknown` for an unknown name and
/// `FilterLookupError::Arity` for an out-of-contract argument count.
pub fn validate_filter(name: &str, arg_count: usize) -> Result<&'static FilterSpec, FilterLookupError> {
    let spec = filter_spec(name).ok_or(FilterLookupError::Unknown)?;
    if arg_count < spec.min_args || arg_count > spec.max_args {
        return Err(FilterLookupError::Arity { spec });
    }
    Ok(spec)
}

/// Why a filter reference is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterLookupError {
    /// No built-in filter matches this name or any alias.
    Unknown,
    /// The filter exists but the argument count is outside its contract.
    Arity { spec: &'static FilterSpec },
}

/// Serialize the table for cross-host conformance checks.
pub fn to_json() -> String {
    let entries: Vec<String> = FILTERS
        .iter()
        .map(|spec| {
            let aliases: Vec<String> =
                spec.aliases.iter().map(|a| format!("\"{a}\"")).collect();
            let capability = match spec.capability {
                FilterCapability::Pure => "pure",
                FilterCapability::SafeOutput => "safe-output",
            };
            format!(
                "{{\"name\":\"{}\",\"aliases\":[{}],\"minArgs\":{},\"maxArgs\":{},\"capability\":\"{}\"}}",
                spec.name,
                aliases.join(","),
                spec.min_args,
                spec.max_args,
                capability
            )
        })
        .collect();
    format!("[{}]", entries.join(","))
}
