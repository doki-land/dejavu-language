//! Filter system for DejaVu templates

use crate::{
    errors::{DejavuError, DejavuResult, RuntimeError},
    values::{Context, Value},
};
use std::{collections::HashMap, sync::Arc};

/// Filter trait for template filters
pub trait Filter: Send + Sync + std::fmt::Debug {
    /// Apply the filter to a value
    ///
    /// # Arguments
    /// * `value` - The value to filter
    /// * `args` - Filter arguments
    /// * `context` - Template context
    ///
    /// # Returns
    /// The filtered value
    fn apply(&self, value: Value, args: &[Value], context: &Context) -> DejavuResult<Value>;
}

/// Filter registry for managing filters
#[derive(Debug, Clone)]
pub struct FilterRegistry {
    filters: HashMap<String, Arc<dyn Filter>>,
}

impl FilterRegistry {
    /// Create a new filter registry
    pub fn new() -> Self {
        let mut registry = Self { filters: HashMap::new() };
        registry.register_builtin_filters();
        registry
    }

    /// Register a filter
    pub fn register(&mut self, name: &str, filter: Arc<dyn Filter>) {
        self.filters.insert(name.to_string(), filter);
    }

    /// Get a filter by name
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Filter>> {
        self.filters.get(name)
    }

    /// Apply a filter to a value
    pub fn apply_filter(&self, name: &str, value: Value, args: &[Value], context: &Context) -> DejavuResult<Value> {
        match self.get(name) {
            Some(filter) => filter.apply(value, args, context),
            None => Err(DejavuError::RuntimeError(RuntimeError::FilterNotFound(name.to_string()))),
        }
    }
}

impl Default for FilterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Built-in filters

/// Uppercase filter
#[derive(Debug)]
pub struct UppercaseFilter;

impl Filter for UppercaseFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            _ => Ok(value),
        }
    }
}

/// Lowercase filter
#[derive(Debug)]
pub struct LowercaseFilter;

impl Filter for LowercaseFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::String(s) => Ok(Value::String(s.to_lowercase())),
            _ => Ok(value),
        }
    }
}

/// Trim filter
#[derive(Debug)]
pub struct TrimFilter;

impl Filter for TrimFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::String(s) => Ok(Value::String(s.trim().to_string())),
            _ => Ok(value),
        }
    }
}

/// Slice filter
#[derive(Debug)]
pub struct SliceFilter;

impl Filter for SliceFilter {
    fn apply(&self, value: Value, args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::String(s) => {
                let start = if args.len() > 0 {
                    match args[0] {
                        Value::Integer(i) => i.max(0) as usize,
                        _ => 0,
                    }
                } else {
                    0
                };

                let end = if args.len() > 1 {
                    match args[1] {
                        Value::Integer(i) => i.max(start as i64) as usize,
                        _ => s.len(),
                    }
                } else {
                    s.len()
                };

                Ok(Value::String(s[start..end.min(s.len())].to_string()))
            }
            _ => Ok(value),
        }
    }
}

/// Round filter
#[derive(Debug)]
pub struct RoundFilter;

impl Filter for RoundFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::Decimal(f) => Ok(Value::Integer(f.round() as i64)),
            Value::Integer(i) => Ok(Value::Integer(i)),
            _ => Ok(value),
        }
    }
}

/// Floor filter
#[derive(Debug)]
pub struct FloorFilter;

impl Filter for FloorFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::Decimal(f) => Ok(Value::Integer(f.floor() as i64)),
            Value::Integer(i) => Ok(Value::Integer(i)),
            _ => Ok(value),
        }
    }
}

/// Ceil filter
#[derive(Debug)]
pub struct CeilFilter;

impl Filter for CeilFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::Decimal(f) => Ok(Value::Integer(f.ceil() as i64)),
            Value::Integer(i) => Ok(Value::Integer(i)),
            _ => Ok(value),
        }
    }
}

/// Bool filter
#[derive(Debug)]
pub struct BoolFilter;

impl Filter for BoolFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        Ok(Value::Bool(value.is_truthy()))
    }
}

/// Default filter
#[derive(Debug)]
pub struct DefaultFilter;

impl Filter for DefaultFilter {
    fn apply(&self, value: Value, args: &[Value], _context: &Context) -> DejavuResult<Value> {
        if value.is_truthy() {
            Ok(value)
        } else {
            if let Some(default_value) = args.get(0) { Ok(default_value.clone()) } else { Ok(Value::String("".to_string())) }
        }
    }
}

/// Length filter
#[derive(Debug)]
pub struct LengthFilter;

impl Filter for LengthFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::String(s) => Ok(Value::Integer(s.len() as i64)),
            Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
            Value::Object(obj) => Ok(Value::Integer(obj.len() as i64)),
            _ => Ok(Value::Integer(0)),
        }
    }
}

/// Join filter
#[derive(Debug)]
pub struct JoinFilter;

impl Filter for JoinFilter {
    fn apply(&self, value: Value, args: &[Value], _context: &Context) -> DejavuResult<Value> {
        let separator = args
            .get(0)
            .and_then(|v| match v {
                Value::String(s) => Some(s.as_str()),
                _ => None,
            })
            .unwrap_or("");
        match value {
            Value::Array(arr) => {
                let joined: String = arr
                    .iter()
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        _ => "".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(separator);
                Ok(Value::String(joined))
            }
            _ => Ok(Value::String("".to_string())),
        }
    }
}

/// Replace filter
#[derive(Debug)]
pub struct ReplaceFilter;

impl Filter for ReplaceFilter {
    fn apply(&self, value: Value, args: &[Value], _context: &Context) -> DejavuResult<Value> {
        if let Value::String(s) = value {
            if args.len() >= 2 {
                let old = match &args[0] {
                    Value::String(s) => s.as_str(),
                    _ => "",
                };
                let new = match &args[1] {
                    Value::String(s) => s.as_str(),
                    _ => "",
                };
                Ok(Value::String(s.replace(old, new)))
            } else {
                Ok(Value::String(s))
            }
        } else {
            Ok(value)
        }
    }
}

/// Split filter
#[derive(Debug)]
pub struct SplitFilter;

impl Filter for SplitFilter {
    fn apply(&self, value: Value, args: &[Value], _context: &Context) -> DejavuResult<Value> {
        if let Value::String(s) = value {
            let separator = args
                .get(0)
                .and_then(|v| match v {
                    Value::String(s) => Some(s.as_str()),
                    _ => None,
                })
                .unwrap_or("")
                .to_string();
            let parts: Vec<Value> = s.split(&separator).map(|part| Value::String(part.to_string())).collect();
            Ok(Value::Array(parts))
        } else {
            Ok(value)
        }
    }
}

/// Capitalize filter
#[derive(Debug)]
pub struct CapitalizeFilter;

impl Filter for CapitalizeFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::String(s) => {
                let mut chars = s.chars();
                match chars.next() {
                    Some(first) => Ok(Value::String(first.to_uppercase().collect::<String>() + chars.as_str())),
                    None => Ok(Value::String(s)),
                }
            }
            _ => Ok(value),
        }
    }
}

/// Title filter
#[derive(Debug)]
pub struct TitleFilter;

impl Filter for TitleFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::String(s) => {
                let title = s
                    .split_whitespace()
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                            None => word.to_string(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                Ok(Value::String(title))
            }
            _ => Ok(value),
        }
    }
}

/// Strip tags filter
#[derive(Debug)]
pub struct StripTagsFilter;

impl Filter for StripTagsFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::String(s) => {
                // Simple HTML tag stripping
                let stripped: String = s
                    .chars()
                    .fold((String::new(), false), |(mut result, in_tag), c| match c {
                        '<' => (result, true),
                        '>' => (result, false),
                        _ if in_tag => (result, true),
                        _ => (result + &c.to_string(), false),
                    })
                    .0;
                Ok(Value::String(stripped))
            }
            _ => Ok(value),
        }
    }
}

/// Absolute value filter
#[derive(Debug)]
pub struct AbsFilter;

impl Filter for AbsFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::Integer(i) => Ok(Value::Integer(i.abs())),
            Value::Decimal(f) => Ok(Value::Decimal(f.abs())),
            _ => Ok(value),
        }
    }
}

/// Not filter
#[derive(Debug)]
pub struct NotFilter;

impl Filter for NotFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        Ok(Value::Bool(!value.is_truthy()))
    }
}

/// First filter
#[derive(Debug)]
pub struct FirstFilter;

impl Filter for FirstFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::Array(arr) => Ok(arr.first().cloned().unwrap_or(Value::Null)),
            _ => Ok(value),
        }
    }
}

/// Last filter
#[derive(Debug)]
pub struct LastFilter;

impl Filter for LastFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::Array(arr) => Ok(arr.last().cloned().unwrap_or(Value::Null)),
            _ => Ok(value),
        }
    }
}

/// Sort filter
#[derive(Debug)]
pub struct SortFilter;

impl Filter for SortFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::Array(mut arr) => {
                arr.sort_by(|a, b| {
                    let a_str = match a {
                        Value::String(s) => s.as_str(),
                        _ => "",
                    };
                    let b_str = match b {
                        Value::String(s) => s.as_str(),
                        _ => "",
                    };
                    a_str.cmp(b_str)
                });
                Ok(Value::Array(arr))
            }
            _ => Ok(value),
        }
    }
}

/// Reverse filter
#[derive(Debug)]
pub struct ReverseFilter;

impl Filter for ReverseFilter {
    fn apply(&self, value: Value, _args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::Array(mut arr) => {
                arr.reverse();
                Ok(Value::Array(arr))
            }
            Value::String(s) => Ok(Value::String(s.chars().rev().collect())),
            _ => Ok(value),
        }
    }
}

/// Format filter
#[derive(Debug)]
pub struct FormatFilter;

impl Filter for FormatFilter {
    fn apply(&self, value: Value, args: &[Value], _context: &Context) -> DejavuResult<Value> {
        match value {
            Value::String(format_str) => {
                let mut formatted = format_str;

                // Simple format implementation: {0}, {1}, etc.
                for (i, arg) in args.iter().enumerate() {
                    let placeholder = format!("{{{}}}", i);
                    let arg_str = match arg {
                        Value::String(s) => s.clone(),
                        Value::Integer(i) => i.to_string(),
                        Value::Decimal(f) => f.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => "".to_string(),
                    };
                    formatted = formatted.replace(&placeholder, &arg_str);
                }

                Ok(Value::String(formatted))
            }
            _ => Ok(value),
        }
    }
}

/// Implement register_builtin_filters after all filter structs are defined
impl FilterRegistry {
    fn register_builtin_filters(&mut self) {
        // String filters
        self.register("uppercase", Arc::new(UppercaseFilter));
        self.register("lowercase", Arc::new(LowercaseFilter));
        self.register("trim", Arc::new(TrimFilter));
        self.register("slice", Arc::new(SliceFilter));
        self.register("default", Arc::new(DefaultFilter));
        self.register("length", Arc::new(LengthFilter));
        self.register("join", Arc::new(JoinFilter));
        self.register("replace", Arc::new(ReplaceFilter));
        self.register("split", Arc::new(SplitFilter));
        self.register("capitalize", Arc::new(CapitalizeFilter));
        self.register("title", Arc::new(TitleFilter));
        self.register("striptags", Arc::new(StripTagsFilter));
        self.register("format", Arc::new(FormatFilter));

        // Numeric filters
        self.register("round", Arc::new(RoundFilter));
        self.register("floor", Arc::new(FloorFilter));
        self.register("ceil", Arc::new(CeilFilter));
        self.register("abs", Arc::new(AbsFilter));

        // Boolean filters
        self.register("bool", Arc::new(BoolFilter));
        self.register("not", Arc::new(NotFilter));

        // List filters
        self.register("first", Arc::new(FirstFilter));
        self.register("last", Arc::new(LastFilter));
        self.register("sort", Arc::new(SortFilter));
        self.register("reverse", Arc::new(ReverseFilter));
    }
}
