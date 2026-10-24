//! Language-neutral description of the built-in `filter`, `function` and
//! `test` namespaces.
//!
//! Every host (Rust JIT, WASM, TypeScript AOT runtime) must agree on the same
//! names, aliases and arity. This module is the Rust source of truth: the JIT
//! renderer and the AOT compiler validate against it, and
//! `examples/filter_registry.rs` dumps the same tables for cross-host checks.
//!
//! This is intentionally a static table, not a runtime registry: no host may
//! add entries that the other hosts do not implement, and arbitrary
//! `Expr.Call` / host methods stay closed — only names listed here execute.

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

// ---------------------------------------------------------------------------
// function namespace
// ---------------------------------------------------------------------------

/// One built-in function contract.
///
/// Functions are pure: they never read the context, never touch the host and
/// never produce output escaping. A call is only valid when the callee is a
/// bare identifier resolving to a spec in this table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FunctionSpec {
    /// Canonical name used by dispatch.
    pub name: &'static str,
    /// Accepted alternative spellings, all dispatching to `name`.
    pub aliases: &'static [&'static str],
    /// Minimum number of arguments.
    pub min_args: usize,
    /// Maximum number of arguments.
    pub max_args: usize,
}

const fn function_entry(
    name: &'static str,
    aliases: &'static [&'static str],
    min_args: usize,
    max_args: usize,
) -> FunctionSpec {
    FunctionSpec { name, aliases, min_args, max_args }
}

/// The complete built-in function set, in contract order.
pub const FUNCTIONS: &[FunctionSpec] = &[
    function_entry("range", &[], 2, 3),
    function_entry("min", &[], 2, 2),
    function_entry("max", &[], 2, 2),
];

/// Look up a function by canonical name or alias.
pub fn function_spec(name: &str) -> Option<&'static FunctionSpec> {
    FUNCTIONS.iter().find(|spec| spec.name == name || spec.aliases.contains(&name))
}

/// Validate a function name and argument count.
pub fn validate_function(name: &str, arg_count: usize) -> Result<&'static FunctionSpec, FunctionLookupError> {
    let spec = function_spec(name).ok_or(FunctionLookupError::Unknown)?;
    if arg_count < spec.min_args || arg_count > spec.max_args {
        return Err(FunctionLookupError::Arity { spec });
    }
    Ok(spec)
}

/// Why a function reference is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionLookupError {
    /// No built-in function matches this name or any alias.
    Unknown,
    /// The function exists but the argument count is outside its contract.
    Arity { spec: &'static FunctionSpec },
}

// ---------------------------------------------------------------------------
// test namespace
// ---------------------------------------------------------------------------

/// One built-in predicate contract.
///
/// Tests are pure predicates over the evaluated left-hand value; they never
/// read the context and never render output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestSpec {
    /// Canonical name used by dispatch.
    pub name: &'static str,
    /// Accepted alternative spellings, all dispatching to `name`.
    pub aliases: &'static [&'static str],
    /// Minimum number of arguments.
    pub min_args: usize,
    /// Maximum number of arguments.
    pub max_args: usize,
}

const fn test_entry(name: &'static str, aliases: &'static [&'static str]) -> TestSpec {
    TestSpec { name, aliases, min_args: 0, max_args: 0 }
}

/// The complete built-in test set, in contract order.
pub const TESTS: &[TestSpec] = &[
    test_entry("null", &["none"]),
    test_entry("truthy", &[]),
    test_entry("empty", &[]),
    test_entry("array", &[]),
    test_entry("object", &[]),
    test_entry("string", &[]),
    test_entry("number", &[]),
    test_entry("boolean", &[]),
    test_entry("even", &[]),
    test_entry("odd", &[]),
];

/// Look up a test by canonical name or alias.
pub fn test_spec(name: &str) -> Option<&'static TestSpec> {
    TESTS.iter().find(|spec| spec.name == name || spec.aliases.contains(&name))
}

/// Validate a test name and argument count.
pub fn validate_test(name: &str, arg_count: usize) -> Result<&'static TestSpec, TestLookupError> {
    let spec = test_spec(name).ok_or(TestLookupError::Unknown)?;
    if arg_count < spec.min_args || arg_count > spec.max_args {
        return Err(TestLookupError::Arity { spec });
    }
    Ok(spec)
}

/// Why a test reference is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestLookupError {
    /// No built-in test matches this name or any alias.
    Unknown,
    /// The test exists but the argument count is outside its contract.
    Arity { spec: &'static TestSpec },
}

// ---------------------------------------------------------------------------
// Cross-host dump
// ---------------------------------------------------------------------------

fn json_string(value: &str) -> String {
    serde_json::to_string(value).expect("str always serializes")
}

fn json_aliases(aliases: &[&str]) -> String {
    let items: Vec<String> = aliases.iter().map(|alias| json_string(alias)).collect();
    format!("[{}]", items.join(","))
}

/// Serialize all three namespaces for cross-host conformance checks.
///
/// Shape: `{"filters":[...],"functions":[...],"tests":[...]}`. Field names and
/// ordering are part of the conformance contract.
pub fn to_json() -> String {
    let filters: Vec<String> = FILTERS
        .iter()
        .map(|spec| {
            let capability = match spec.capability {
                FilterCapability::Pure => "pure",
                FilterCapability::SafeOutput => "safe-output",
            };
            format!(
                "{{\"name\":{},\"aliases\":{},\"minArgs\":{},\"maxArgs\":{},\"capability\":{}}}",
                json_string(spec.name),
                json_aliases(spec.aliases),
                spec.min_args,
                spec.max_args,
                json_string(capability)
            )
        })
        .collect();
    let functions: Vec<String> = FUNCTIONS
        .iter()
        .map(|spec| {
            format!(
                "{{\"name\":{},\"aliases\":{},\"minArgs\":{},\"maxArgs\":{}}}",
                json_string(spec.name),
                json_aliases(spec.aliases),
                spec.min_args,
                spec.max_args
            )
        })
        .collect();
    let tests: Vec<String> = TESTS
        .iter()
        .map(|spec| {
            format!(
                "{{\"name\":{},\"aliases\":{},\"minArgs\":{},\"maxArgs\":{}}}",
                json_string(spec.name),
                json_aliases(spec.aliases),
                spec.min_args,
                spec.max_args
            )
        })
        .collect();
    format!(
        "{{\"filters\":[{}],\"functions\":[{}],\"tests\":[{}]}}",
        filters.join(","),
        functions.join(","),
        tests.join(",")
    )
}
