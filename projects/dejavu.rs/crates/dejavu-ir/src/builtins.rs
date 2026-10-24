//! Runtime implementations for the `function` and `test` namespaces.
//!
//! Names, aliases and arity come from [`crate::registry`]; this module only
//! evaluates already-validated calls. The same semantics are implemented by the
//! TypeScript AOT runtime (`@doki-land/dejavu-engine/runtime`) and by the
//! optional Python / C# hosts.
//!
//! Functions are pure and closed: an `Expr.Call` whose callee is not a bare
//! identifier resolving to a registry entry is rejected, never dispatched to
//! the host. Invalid argument *values* follow the value-model rule used by the
//! arithmetic operators: the result is `null`, not an error.

use serde_json::Value;

use crate::IrError;
use crate::codes;
use crate::diag_render;
use crate::registry::{CapabilitySet, FunctionLookupError, TestLookupError, validate_function, validate_test};

/// Maximum number of elements `range` may produce.
///
/// Guards against runaway allocations from dynamic bounds; exceeding the cap
/// follows the invalid-argument rule and yields `null`.
const RANGE_LIMIT: usize = 10_000;

/// Evaluate a built-in function call.
pub fn call_function(name: &str, args: &[Value], capabilities: CapabilitySet) -> Result<Value, IrError> {
    let spec = validate_function(name, args.len(), capabilities).map_err(|err| match err {
        FunctionLookupError::Unknown => {
            diag_render(codes::UNKNOWN_FUNCTION).arg("function", name)
        }
        FunctionLookupError::Arity { spec } => diag_render(codes::FUNCTION_ARITY)
            .arg("function", spec.name)
            .arg("minArgs", spec.min_args)
            .arg("maxArgs", spec.max_args)
            .arg("got", args.len()),
        FunctionLookupError::NotAllowed { spec } => {
            crate::render::capability_denied(spec.name, spec.capability)
        }
    })?;
    Ok(match spec.name {
        "range" => range(args),
        "min" => numeric_pair(args, true),
        "max" => numeric_pair(args, false),
        _ => Value::Null,
    })
}

/// Evaluate a built-in test against an already-evaluated value.
///
/// `negated` is the `is not` form. Callers must pass `negated=false` when the
/// source had no `not` marker.
pub fn call_test(
    name: &str,
    value: &Value,
    args: &[Value],
    negated: bool,
    capabilities: CapabilitySet,
) -> Result<bool, IrError> {
    let spec = validate_test(name, args.len(), capabilities).map_err(|err| match err {
        TestLookupError::Unknown => diag_render(codes::UNKNOWN_TEST).arg("test", name),
        TestLookupError::Arity { spec } => diag_render(codes::TEST_ARITY)
            .arg("test", spec.name)
            .arg("minArgs", spec.min_args)
            .arg("maxArgs", spec.max_args)
            .arg("got", args.len()),
        TestLookupError::NotAllowed { spec } => crate::render::capability_denied(spec.name, spec.capability),
    })?;
    let result = match spec.name {
        "null" => value.is_null(),
        "truthy" => is_truthy(value),
        "empty" => match value {
            Value::String(s) => s.is_empty(),
            Value::Array(a) => a.is_empty(),
            Value::Object(o) => o.is_empty(),
            _ => false,
        },
        "array" => value.is_array(),
        "object" => value.is_object(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "even" => integral(value).is_some_and(|n| n % 2 == 0),
        "odd" => integral(value).is_some_and(|n| n % 2 != 0),
        _ => false,
    };
    Ok(result != negated)
}

/// `range(start, stop, step=1)` over integers.
fn range(args: &[Value]) -> Value {
    let Some(start) = integral(&args[0]) else { return Value::Null };
    let Some(stop) = integral(&args[1]) else { return Value::Null };
    let step = match args.get(2) {
        Some(value) => match integral(value) {
            Some(0) => return Value::Null,
            Some(n) => n,
            None => return Value::Null,
        },
        None => 1,
    };
    let mut out = Vec::new();
    let mut current = start;
    while (step > 0 && current < stop) || (step < 0 && current > stop) {
        if out.len() >= RANGE_LIMIT {
            return Value::Null;
        }
        out.push(Value::Number(current.into()));
        current = match current.checked_add(step) {
            Some(next) => next,
            None => break,
        };
    }
    Value::Array(out)
}

/// `min(a, b)` / `max(a, b)` over finite numbers.
fn numeric_pair(args: &[Value], min: bool) -> Value {
    let (Some(a), Some(b)) = (args[0].as_f64(), args[1].as_f64()) else { return Value::Null };
    if !a.is_finite() || !b.is_finite() {
        return Value::Null;
    }
    let picked = if min { a.min(b) } else { a.max(b) };
    // Integral results must stay integers so their string form matches JS (`4`,
    // not `4.0`) across hosts.
    if picked.fract() == 0.0 && picked >= i64::MIN as f64 && picked <= i64::MAX as f64 {
        return Value::Number((picked as i64).into());
    }
    serde_json::Number::from_f64(picked).map(Value::Number).unwrap_or(Value::Null)
}

/// Integral numeric value, or `None` for non-numbers / non-integers.
fn integral(value: &Value) -> Option<i64> {
    let n = value.as_f64()?;
    if !n.is_finite() || n.fract() != 0.0 {
        return None;
    }
    if n < i64::MIN as f64 || n > i64::MAX as f64 {
        return None;
    }
    Some(n as i64)
}

/// 1.0 truthiness — same table as the renderer and the value-model contract.
fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}
