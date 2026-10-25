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
//!
//! Every entry declares the [`Capability`] it needs. A host may grant a subset
//! through [`CapabilitySet`] (render / codegen options); an entry outside that
//! subset is rejected with `dejavu.template.capability-not-allowed` instead of
//! executing. Without an explicit grant the default is [`CapabilitySet::all`].

use dejavu_diagnostic::{Diagnostic, Severity, Stage, codes};
use serde::{Deserialize, Serialize};

/// A capability a registry entry needs from its host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    /// Ordinary value transformation. The interpolation pipeline still escapes.
    Pure,
    /// The entry owns the output escaping (`escape`/`e`) or explicitly opts out
    /// of escaping (`safe`/`raw`). The pipeline must not escape again.
    SafeOutput,
}

impl Default for Capability {
    /// An entry that omits `capability` never owns its output escaping.
    fn default() -> Self {
        Capability::Pure
    }
}

impl Capability {
    /// Stable wire name used by options and cross-host registry dumps.
    pub const fn as_str(self) -> &'static str {
        match self {
            Capability::Pure => "pure",
            Capability::SafeOutput => "safe-output",
        }
    }

    /// Parse a wire name; returns `None` for anything not in the contract.
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "pure" => Some(Capability::Pure),
            "safe-output" => Some(Capability::SafeOutput),
            _ => None,
        }
    }
}

/// The capabilities a host grants to registry entries.
///
/// `Default` grants everything, which is the behaviour hosts had before the
/// allowlist existed. Passing an explicit list (for example `["pure"]`) closes
/// the `safe-output` entries for that render / compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilitySet {
    pure: bool,
    safe_output: bool,
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::all()
    }
}

impl CapabilitySet {
    /// Grant every capability in the contract.
    pub const fn all() -> Self {
        Self { pure: true, safe_output: true }
    }

    /// Grant no capability; every registry entry is rejected.
    pub const fn none() -> Self {
        Self { pure: false, safe_output: false }
    }

    /// Build from wire names, granting exactly the listed capabilities.
    ///
    /// Returns the first unknown name so hosts can report it as a configuration
    /// error instead of silently narrowing or widening the set.
    pub fn from_names(names: &[String]) -> Result<Self, String> {
        let mut set = Self::none();
        for name in names {
            match Capability::parse(name) {
                Some(Capability::Pure) => set.pure = true,
                Some(Capability::SafeOutput) => set.safe_output = true,
                None => return Err(name.clone()),
            }
        }
        Ok(set)
    }

    /// Whether `capability` is granted.
    pub const fn allows(self, capability: Capability) -> bool {
        match capability {
            Capability::Pure => self.pure,
            Capability::SafeOutput => self.safe_output,
        }
    }
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
    /// Capability the host must grant for this filter to run.
    pub capability: Capability,
}

const fn spec(
    name: &'static str,
    aliases: &'static [&'static str],
    min_args: usize,
    max_args: usize,
    capability: Capability,
) -> FilterSpec {
    FilterSpec { name, aliases, min_args, max_args, capability }
}

/// The complete built-in filter set, in contract order.
pub const FILTERS: &[FilterSpec] = &[
    spec("uppercase", &["upper"], 0, 0, Capability::Pure),
    spec("lowercase", &["lower"], 0, 0, Capability::Pure),
    spec("trim", &[], 0, 0, Capability::Pure),
    spec("default", &[], 1, 1, Capability::Pure),
    spec("length", &[], 0, 0, Capability::Pure),
    spec("join", &[], 0, 1, Capability::Pure),
    spec("replace", &[], 2, 2, Capability::Pure),
    spec("escape", &["e"], 0, 0, Capability::SafeOutput),
    spec("safe", &["raw"], 0, 0, Capability::SafeOutput),
    spec("date", &[], 0, 1, Capability::Pure),
    spec("slug", &[], 0, 0, Capability::Pure),
];

/// Look up a filter by canonical name or alias.
pub fn filter_spec(name: &str) -> Option<&'static FilterSpec> {
    FILTERS
        .iter()
        .find(|spec| spec.name == name || spec.aliases.contains(&name))
}

/// Validate a filter name, argument count and host-granted capabilities.
///
/// Returns the canonical spec on success, or the violated rule:
/// `FilterLookupError::Unknown` for an unknown name,
/// `FilterLookupError::Arity` for an out-of-contract argument count and
/// `FilterLookupError::NotAllowed` when the host did not grant the filter's
/// capability.
pub fn validate_filter(
    name: &str,
    arg_count: usize,
    capabilities: CapabilitySet,
) -> Result<&'static FilterSpec, FilterLookupError> {
    let spec = filter_spec(name).ok_or(FilterLookupError::Unknown)?;
    if arg_count < spec.min_args || arg_count > spec.max_args {
        return Err(FilterLookupError::Arity { spec });
    }
    if !capabilities.allows(spec.capability) {
        return Err(FilterLookupError::NotAllowed { spec });
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
    /// The host did not grant the filter's [`Capability`].
    NotAllowed { spec: &'static FilterSpec },
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
    /// Capability the host must grant for this function to run.
    pub capability: Capability,
}

const fn function_entry(
    name: &'static str,
    aliases: &'static [&'static str],
    min_args: usize,
    max_args: usize,
    capability: Capability,
) -> FunctionSpec {
    FunctionSpec { name, aliases, min_args, max_args, capability }
}

/// The complete built-in function set, in contract order.
pub const FUNCTIONS: &[FunctionSpec] = &[
    function_entry("range", &[], 2, 3, Capability::Pure),
    function_entry("min", &[], 2, 2, Capability::Pure),
    function_entry("max", &[], 2, 2, Capability::Pure),
];

/// Look up a function by canonical name or alias.
pub fn function_spec(name: &str) -> Option<&'static FunctionSpec> {
    FUNCTIONS.iter().find(|spec| spec.name == name || spec.aliases.contains(&name))
}

/// Validate a function name, argument count and host-granted capabilities.
pub fn validate_function(
    name: &str,
    arg_count: usize,
    capabilities: CapabilitySet,
) -> Result<&'static FunctionSpec, FunctionLookupError> {
    let spec = function_spec(name).ok_or(FunctionLookupError::Unknown)?;
    if arg_count < spec.min_args || arg_count > spec.max_args {
        return Err(FunctionLookupError::Arity { spec });
    }
    if !capabilities.allows(spec.capability) {
        return Err(FunctionLookupError::NotAllowed { spec });
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
    /// The host did not grant the function's [`Capability`].
    NotAllowed { spec: &'static FunctionSpec },
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
    /// Capability the host must grant for this test to run.
    pub capability: Capability,
}

const fn test_entry(name: &'static str, aliases: &'static [&'static str]) -> TestSpec {
    TestSpec { name, aliases, min_args: 0, max_args: 0, capability: Capability::Pure }
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

/// Validate a test name, argument count and host-granted capabilities.
pub fn validate_test(
    name: &str,
    arg_count: usize,
    capabilities: CapabilitySet,
) -> Result<&'static TestSpec, TestLookupError> {
    let spec = test_spec(name).ok_or(TestLookupError::Unknown)?;
    if arg_count < spec.min_args || arg_count > spec.max_args {
        return Err(TestLookupError::Arity { spec });
    }
    if !capabilities.allows(spec.capability) {
        return Err(TestLookupError::NotAllowed { spec });
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
    /// The host did not grant the test's [`Capability`].
    NotAllowed { spec: &'static TestSpec },
}

// ---------------------------------------------------------------------------
// host-supplied registry entries
// ---------------------------------------------------------------------------

/// Namespace a host-supplied entry extends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Namespace {
    /// Filter namespace, applied through `|>`.
    Filter,
    /// Function namespace, reachable only through a bare identifier call.
    Function,
    /// Predicate namespace, used by `is` / `is not`.
    Test,
}

impl Namespace {
    /// Stable wire name used by options and cross-host registry dumps.
    pub const fn as_str(self) -> &'static str {
        match self {
            Namespace::Filter => "filter",
            Namespace::Function => "function",
            Namespace::Test => "test",
        }
    }
}

/// One host-supplied registry entry description.
///
/// The shape is identical to the built-in tables (`name` / `aliases` /
/// `minArgs` / `maxArgs` / `capability`) plus the `namespace` discriminator.
/// Descriptions are language-neutral: every host must consume the same list,
/// and the implementation is supplied separately by that host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomEntry {
    /// Namespace this entry extends.
    pub namespace: Namespace,
    /// Canonical name used by dispatch.
    pub name: String,
    /// Accepted alternative spellings, all dispatching to `name`.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Minimum number of explicit arguments.
    #[serde(default)]
    pub min_args: usize,
    /// Maximum number of explicit arguments.
    #[serde(default)]
    pub max_args: usize,
    /// Capability the host must grant for this entry to run.
    #[serde(default)]
    pub capability: Capability,
}

// ---------------------------------------------------------------------------
// Custom entry validation
// ---------------------------------------------------------------------------

/// Whether `name` already resolves inside the built-in table of `namespace`.
fn builtin_matches(namespace: Namespace, name: &str) -> bool {
    match namespace {
        Namespace::Filter => FILTERS.iter().any(|spec| spec.name == name || spec.aliases.contains(&name)),
        Namespace::Function => FUNCTIONS.iter().any(|spec| spec.name == name || spec.aliases.contains(&name)),
        Namespace::Test => TESTS.iter().any(|spec| spec.name == name || spec.aliases.contains(&name)),
    }
}

fn conflict_diagnostic(namespace: Namespace, name: &str) -> Diagnostic {
    Diagnostic::new(Severity::Error, codes::REGISTRY_CONFLICT, Stage::Host)
        .message(format!(
            "registry entry `{name}` conflicts with an existing `{}` entry",
            namespace.as_str()
        ))
        .arg("namespace", namespace.as_str())
        .arg("name", name)
}

/// Validate host-supplied entry descriptions before they are used.
///
/// A name (canonical or alias) must not resolve to a built-in entry, and two
/// custom entries in the same namespace must not share any spelling. Returning
/// the diagnostic means the registration itself is rejected; templates never
/// see a half-merged registry. Built-in arity and capability rules continue to
/// apply afterwards.
pub fn validate_custom_entries(entries: &[CustomEntry]) -> Result<(), Diagnostic> {
    for entry in entries {
        for name in std::iter::once(entry.name.as_str()).chain(entry.aliases.iter().map(String::as_str)) {
            if builtin_matches(entry.namespace, name) {
                return Err(conflict_diagnostic(entry.namespace, name));
            }
        }
    }
    for (index, left) in entries.iter().enumerate() {
        for right in entries.iter().skip(index + 1) {
            if left.namespace != right.namespace {
                continue;
            }
            for name in std::iter::once(right.name.as_str()).chain(right.aliases.iter().map(String::as_str)) {
                if left.name == name || left.aliases.iter().any(|alias| alias == name) {
                    return Err(conflict_diagnostic(right.namespace, name));
                }
            }
        }
    }
    Ok(())
}

/// Look up a custom entry by canonical name or alias inside one namespace.
pub fn custom_entry<'a>(
    entries: &'a [CustomEntry],
    namespace: Namespace,
    name: &str,
) -> Option<&'a CustomEntry> {
    entries
        .iter()
        .find(|entry| entry.namespace == namespace && (entry.name == name || entry.aliases.iter().any(|alias| alias == name)))
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
    to_json_with_custom(&[])
}

/// Serialize built-ins merged with host-supplied entries.
///
/// Custom entries are appended to their namespace table in input order and use
/// the exact same object shape, so the conformance gate can compare the merged
/// description across hosts without special-casing custom entries.
pub fn to_json_with_custom(custom: &[CustomEntry]) -> String {
    let filters = merged_table(FILTERS.iter().map(|spec| {
        entry_json(spec.name, spec.aliases, spec.min_args, spec.max_args, spec.capability.as_str())
    }), custom, Namespace::Filter);
    let functions = merged_table(FUNCTIONS.iter().map(|spec| {
        entry_json(spec.name, spec.aliases, spec.min_args, spec.max_args, spec.capability.as_str())
    }), custom, Namespace::Function);
    let tests = merged_table(TESTS.iter().map(|spec| {
        entry_json(spec.name, spec.aliases, spec.min_args, spec.max_args, spec.capability.as_str())
    }), custom, Namespace::Test);
    format!(
        "{{\"filters\":[{}],\"functions\":[{}],\"tests\":[{}]}}",
        filters.join(","),
        functions.join(","),
        tests.join(",")
    )
}

fn merged_table(
    builtins: impl Iterator<Item = String>,
    custom: &[CustomEntry],
    namespace: Namespace,
) -> Vec<String> {
    let mut table: Vec<String> = builtins.collect();
    table.extend(custom.iter().filter(|entry| entry.namespace == namespace).map(custom_entry_json));
    table
}

fn entry_json(name: &str, aliases: &[&str], min_args: usize, max_args: usize, capability: &str) -> String {
    format!(
        "{{\"name\":{},\"aliases\":{},\"minArgs\":{},\"maxArgs\":{},\"capability\":{}}}",
        json_string(name),
        json_aliases(aliases),
        min_args,
        max_args,
        json_string(capability)
    )
}

fn custom_entry_json(entry: &CustomEntry) -> String {
    let aliases: Vec<&str> = entry.aliases.iter().map(String::as_str).collect();
    entry_json(&entry.name, &aliases, entry.min_args, entry.max_args, entry.capability.as_str())
}
