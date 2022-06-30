//! Dejavu value types for template rendering

use crate::{
    TranslationProvider,
    errors::{DejavuError, DejavuResult, RuntimeError},
    types::type_info::DejavuType,
};
use std::collections::{BTreeMap, HashMap};

pub mod filter;
use filter::FilterRegistry;

/// Simple value type for template rendering
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// String value
    String(String),
    /// Integer value
    Integer(i64),
    /// Decimal value
    Decimal(f64),
    /// Boolean value
    Bool(bool),
    /// Array value
    Array(Vec<Value>),
    /// Object value
    Object(std::collections::HashMap<String, Value>),
    /// Function value
    Function(String),
    /// Null value
    Null,
}

impl Value {
    /// Get the type name
    pub fn type_name(&self) -> &str {
        match self {
            Value::String(_) => "string",
            Value::Integer(_) => "integer",
            Value::Decimal(_) => "decimal",
            Value::Bool(_) => "boolean",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Function(_) => "function",
            Value::Null => "null",
        }
    }

    /// Get the DejavuType representation
    pub fn get_type(&self) -> DejavuType {
        match self {
            Value::String(_) => DejavuType::String,
            Value::Integer(_) => DejavuType::Integer,
            Value::Decimal(_) => DejavuType::Decimal,
            Value::Bool(_) => DejavuType::Bool,
            Value::Array(items) => {
                if let Some(first) = items.first() {
                    DejavuType::Array(Box::new(first.get_type()))
                } else {
                    DejavuType::Array(Box::new(DejavuType::Any))
                }
            }
            Value::Object(props) => {
                let mut type_props = std::collections::HashMap::new();
                for (key, value) in props {
                    type_props.insert(key.clone(), value.get_type());
                }
                DejavuType::Object(type_props)
            }
            Value::Function(_) => DejavuType::Function(Vec::new(), Box::new(DejavuType::Any)),
            Value::Null => DejavuType::Null,
        }
    }

    /// Check if the value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::String(s) => !s.is_empty(),
            Value::Integer(i) => *i != 0,
            Value::Decimal(f) => *f != 0.0,
            Value::Bool(b) => *b,
            Value::Array(a) => !a.is_empty(),
            Value::Object(o) => !o.is_empty(),
            Value::Function(_) => true,
            Value::Null => false,
        }
    }

    /// Get a property from an object
    pub fn get_property(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Object(o) => o.get(name),
            _ => None,
        }
    }

    /// Convert to output string
    pub fn to_output_string(&self, _mode: &String) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Integer(i) => i.to_string(),
            Value::Decimal(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Array(a) => format!("{:?}", a),
            Value::Object(o) => format!("{:?}", o),
            Value::Function(f) => f.clone(),
            Value::Null => "".to_string(),
        }
    }

    /// Convert to string
    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Convert to array reference
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        self.to_output_string(&"html".to_string())
    }

    /// Check if this value is of the specified type
    pub fn is_type(&self, expected_type: &DejavuType) -> bool {
        let actual_type = self.get_type();
        actual_type.is_subtype_of(expected_type)
    }

    /// Convert value to the specified type
    pub fn as_type(&self, expected_type: &DejavuType) -> DejavuResult<Value> {
        match (self, expected_type) {
            (_, DejavuType::Any) => Ok(self.clone()),
            (Value::Null, _) => Ok(self.clone()),
            (Value::String(s), DejavuType::String) => Ok(self.clone()),
            (Value::Integer(i), DejavuType::Integer) => Ok(self.clone()),
            (Value::Decimal(f), DejavuType::Decimal) => Ok(self.clone()),
            (Value::Bool(b), DejavuType::Bool) => Ok(self.clone()),
            (Value::Array(a), DejavuType::Array(_)) => Ok(self.clone()),
            (Value::Object(o), DejavuType::Object(_)) => Ok(self.clone()),
            (Value::Integer(i), DejavuType::Decimal) => Ok(Value::Decimal(*i as f64)),
            (Value::Decimal(f), DejavuType::String) => Ok(Value::String(f.to_string())),
            (Value::Integer(i), DejavuType::String) => Ok(Value::String(i.to_string())),
            (Value::Bool(b), DejavuType::String) => Ok(Value::String(b.to_string())),
            _ => Err(DejavuError::RuntimeError(RuntimeError::NotImplemented(format!(
                "Type conversion not supported: {} to {}",
                self.type_name(),
                expected_type.name()
            )))),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_output_string(&"html".to_string()))
    }
}

/// Trait for converting Rust types to Dejavu Value
pub trait IntoValue {
    /// Convert this value to a Dejavu Value
    fn into_value(&self) -> Value;
}

impl IntoValue for String {
    fn into_value(&self) -> Value {
        Value::String(self.clone())
    }
}

impl IntoValue for &str {
    fn into_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl IntoValue for bool {
    fn into_value(&self) -> Value {
        Value::Bool(*self)
    }
}

impl IntoValue for i8 {
    fn into_value(&self) -> Value {
        Value::Integer(*self as i64)
    }
}

impl IntoValue for i16 {
    fn into_value(&self) -> Value {
        Value::Integer(*self as i64)
    }
}

impl IntoValue for i32 {
    fn into_value(&self) -> Value {
        Value::Integer(*self as i64)
    }
}

impl IntoValue for i64 {
    fn into_value(&self) -> Value {
        Value::Integer(*self)
    }
}

impl IntoValue for u8 {
    fn into_value(&self) -> Value {
        Value::Integer(*self as i64)
    }
}

impl IntoValue for u16 {
    fn into_value(&self) -> Value {
        Value::Integer(*self as i64)
    }
}

impl IntoValue for u32 {
    fn into_value(&self) -> Value {
        Value::Integer(*self as i64)
    }
}

impl IntoValue for u64 {
    fn into_value(&self) -> Value {
        Value::Integer(*self as i64)
    }
}

impl IntoValue for f32 {
    fn into_value(&self) -> Value {
        Value::Decimal(*self as f64)
    }
}

impl IntoValue for f64 {
    fn into_value(&self) -> Value {
        Value::Decimal(*self)
    }
}

impl<T: IntoValue> IntoValue for Vec<T> {
    fn into_value(&self) -> Value {
        Value::Array(self.iter().map(|item| item.into_value()).collect())
    }
}

impl<T: IntoValue> IntoValue for Option<T> {
    fn into_value(&self) -> Value {
        match self {
            Some(v) => v.into_value(),
            None => Value::Null,
        }
    }
}

impl IntoValue for Value {
    fn into_value(&self) -> Value {
        self.clone()
    }
}

impl IntoValue for &Value {
    fn into_value(&self) -> Value {
        (*self).clone()
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<&String> for Value {
    fn from(value: &String) -> Self {
        Value::String(value.clone())
    }
}

impl From<&bool> for Value {
    fn from(value: &bool) -> Self {
        Value::Bool(*value)
    }
}

impl From<&i64> for Value {
    fn from(value: &i64) -> Self {
        Value::Integer(*value)
    }
}

impl From<&f64> for Value {
    fn from(value: &f64) -> Self {
        Value::Decimal(*value)
    }
}

impl From<&Vec<String>> for Value {
    fn from(value: &Vec<String>) -> Self {
        Value::Array(value.iter().map(|item| Value::from(item)).collect())
    }
}

impl From<&std::collections::HashMap<String, String>> for Value {
    fn from(value: &std::collections::HashMap<String, String>) -> Self {
        let mut map = std::collections::HashMap::new();
        for (k, v) in value {
            map.insert(k.clone(), Value::from(v));
        }
        Value::Object(map)
    }
}

/// Simple context structure for template rendering
pub struct Context {
    /// Template mode
    pub template_mode: String,
    /// Variables - using HashMap for fast lookup
    pub variables: std::collections::HashMap<String, Value>,
    /// Configuration
    pub config: std::collections::HashMap<String, String>,
    /// Translation provider
    pub translation_provider: Option<Box<dyn TranslationProvider>>,
    /// Filter registry
    pub filter_registry: FilterRegistry,
    /// Type environment
    pub type_env: crate::types::type_info::TypeEnv,
    /// Variable lookup cache for frequently accessed variables
    variable_cache: std::collections::HashMap<String, Value>,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("template_mode", &self.template_mode)
            .field("variables", &self.variables)
            .field("config", &self.config)
            .field("translation_provider", &"<TranslationProvider>")
            .field("filter_registry", &"<FilterRegistry>")
            .field("type_env", &"<TypeEnv>")
            .finish()
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            template_mode: self.template_mode.clone(),
            variables: self.variables.clone(),
            config: self.config.clone(),
            translation_provider: None, // Translation provider is not cloned
            filter_registry: self.filter_registry.clone(),
            type_env: self.type_env.clone(),
            variable_cache: std::collections::HashMap::new(),
        }
    }
}

impl Context {
    /// Create a new context
    pub fn new() -> Self {
        Self {
            template_mode: "html".to_string(),
            variables: std::collections::HashMap::new(),
            config: std::collections::HashMap::new(),
            translation_provider: None,
            filter_registry: FilterRegistry::new(),
            type_env: crate::types::type_info::TypeEnv::new(),
            variable_cache: std::collections::HashMap::new(),
        }
    }

    /// Set a variable
    pub fn set_var(&mut self, name: String, value: Value) {
        self.variables.insert(name.clone(), value.clone());
        // Update variable cache
        self.variable_cache.insert(name.clone(), value.clone());
        // Update type environment
        self.type_env.add_variable(name, value.get_type());
    }

    /// Set a variable from any type
    pub fn set_var_from_any(&mut self, name: String, _value: &dyn std::any::Any) {
        let value = Value::Object(std::collections::HashMap::new());
        self.variables.insert(name.clone(), value.clone());
        self.variable_cache.insert(name.clone(), value.clone());
        self.type_env.add_variable(name, value.get_type());
    }

    /// Set a variable from a serializable value
    pub fn set_var_from_serializable<T: serde::Serialize>(&mut self, name: &str, value: &T) {
        let json_value = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
        let dejavu_value = Self::json_to_value(&json_value);
        self.set_var(name.to_string(), dejavu_value);
    }

    /// Convert serde_json::Value to Dejavu Value
    fn json_to_value(json: &serde_json::Value) -> Value {
        match json {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Decimal(f)
                } else {
                    Value::String(n.to_string())
                }
            }
            serde_json::Value::String(s) => Value::String(s.clone()),
            serde_json::Value::Array(arr) => Value::Array(arr.iter().map(Self::json_to_value).collect()),
            serde_json::Value::Object(obj) => {
                let map: std::collections::HashMap<String, Value> =
                    obj.iter().map(|(k, v)| (k.clone(), Self::json_to_value(v))).collect();
                Value::Object(map)
            }
        }
    }

    /// Get a variable with cache optimization
    pub fn get_var(&self, name: &str) -> Option<&Value> {
        // First check cache
        if let Some(value) = self.variable_cache.get(name) {
            return Some(value);
        }
        // If not in cache, check main variables
        if let Some(value) = self.variables.get(name) {
            // Add to cache for future lookups
            // Note: We can't modify self in a &self method, so we'll rely on set_var to update cache
            return Some(value);
        }
        None
    }

    /// Get a variable (alias for get_var)
    pub fn get(&self, name: &str) -> DejavuResult<Value> {
        self.get_var(name).cloned().ok_or_else(|| DejavuError::RuntimeError(RuntimeError::VariableNotFound(name.to_string())))
    }

    /// Get variable type
    pub fn get_var_type(&self, name: &str) -> Option<&crate::types::type_info::DejavuType> {
        self.type_env.get_variable(name)
    }

    /// Call a function
    pub fn call_function(&self, name: &str, args: &[Value]) -> DejavuResult<Value> {
        match name {
            "range" => {
                // range(start, end, step?)
                let start = if args.len() > 0 {
                    match &args[0] {
                        Value::Integer(i) => *i,
                        _ => 0,
                    }
                } else {
                    0
                };

                let end = if args.len() > 1 {
                    match &args[1] {
                        Value::Integer(i) => *i,
                        _ => start + 1,
                    }
                } else {
                    start + 1
                };

                let step = if args.len() > 2 {
                    match &args[2] {
                        Value::Integer(i) => *i.max(&1),
                        _ => 1,
                    }
                } else {
                    1
                };

                let mut result = Vec::new();
                if start < end && step > 0 {
                    let mut current = start;
                    while current < end {
                        result.push(Value::Integer(current));
                        current += step;
                    }
                } else if start > end && step < 0 {
                    let mut current = start;
                    while current > end {
                        result.push(Value::Integer(current));
                        current += step;
                    }
                }

                Ok(Value::Array(result))
            }
            "len" => {
                // len(value)
                if args.len() > 0 {
                    let value = &args[0];
                    match value {
                        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                        Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
                        Value::Object(obj) => Ok(Value::Integer(obj.len() as i64)),
                        _ => Ok(Value::Integer(0)),
                    }
                } else {
                    Ok(Value::Integer(0))
                }
            }
            "join" => {
                // join(array, separator?)
                if args.len() > 0 {
                    let separator = if args.len() > 1 {
                        match &args[1] {
                            Value::String(s) => s.as_str(),
                            _ => "",
                        }
                    } else {
                        ""
                    };

                    match &args[0] {
                        Value::Array(arr) => {
                            let joined: String = arr.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(separator);
                            Ok(Value::String(joined))
                        }
                        _ => Ok(Value::String("".to_string())),
                    }
                } else {
                    Ok(Value::String("".to_string()))
                }
            }
            "abs" => {
                // abs(value)
                if args.len() > 0 {
                    match &args[0] {
                        Value::Integer(i) => Ok(Value::Integer(i.abs())),
                        Value::Decimal(f) => Ok(Value::Decimal(f.abs())),
                        _ => Ok(args[0].clone()),
                    }
                } else {
                    Ok(Value::Null)
                }
            }
            "max" => {
                // max(values...)
                if args.len() > 0 {
                    let mut max_val = args[0].clone();
                    for arg in &args[1..] {
                        match (max_val.clone(), arg.clone()) {
                            (Value::Integer(a), Value::Integer(b)) if b > a => max_val = Value::Integer(b),
                            (Value::Decimal(a), Value::Decimal(b)) if b > a => max_val = Value::Decimal(b),
                            (Value::Integer(a), Value::Decimal(b)) if b > a as f64 => max_val = Value::Decimal(b),
                            (Value::Decimal(a), Value::Integer(b)) if (b as f64) > a => max_val = Value::Integer(b),
                            _ => {}
                        }
                    }
                    Ok(max_val)
                } else {
                    Ok(Value::Null)
                }
            }
            "min" => {
                // min(values...)
                if args.len() > 0 {
                    let mut min_val = args[0].clone();
                    for arg in &args[1..] {
                        match (min_val.clone(), arg.clone()) {
                            (Value::Integer(a), Value::Integer(b)) if b < a => min_val = Value::Integer(b),
                            (Value::Decimal(a), Value::Decimal(b)) if b < a => min_val = Value::Decimal(b),
                            (Value::Integer(a), Value::Decimal(b)) if b < a as f64 => min_val = Value::Decimal(b),
                            (Value::Decimal(a), Value::Integer(b)) if (b as f64) < a => min_val = Value::Integer(b),
                            _ => {}
                        }
                    }
                    Ok(min_val)
                } else {
                    Ok(Value::Null)
                }
            }
            "sum" => {
                // sum(values...)
                let mut total = 0.0;
                for arg in args {
                    match arg {
                        Value::Integer(i) => total += *i as f64,
                        Value::Decimal(f) => total += *f,
                        _ => {}
                    }
                }
                Ok(Value::Decimal(total))
            }
            "split" => {
                // split(string, separator?)
                if args.len() > 0 {
                    let separator = if args.len() > 1 {
                        match &args[1] {
                            Value::String(s) => s.as_str(),
                            _ => "",
                        }
                    } else {
                        ""
                    };

                    match &args[0] {
                        Value::String(s) => {
                            let parts: Vec<Value> = s.split(separator).map(|part| Value::String(part.to_string())).collect();
                            Ok(Value::Array(parts))
                        }
                        _ => Ok(Value::Array(Vec::new())),
                    }
                } else {
                    Ok(Value::Array(Vec::new()))
                }
            }
            "str" => {
                // str(value)
                if args.len() > 0 { Ok(Value::String(args[0].to_string())) } else { Ok(Value::String("".to_string())) }
            }
            "int" => {
                // int(value)
                if args.len() > 0 {
                    match &args[0] {
                        Value::String(s) => {
                            if let Ok(i) = s.parse::<i64>() {
                                Ok(Value::Integer(i))
                            } else {
                                Ok(Value::Integer(0))
                            }
                        }
                        Value::Decimal(f) => Ok(Value::Integer(*f as i64)),
                        Value::Integer(i) => Ok(Value::Integer(*i)),
                        Value::Bool(b) => Ok(Value::Integer(if *b { 1 } else { 0 })),
                        _ => Ok(Value::Integer(0)),
                    }
                } else {
                    Ok(Value::Integer(0))
                }
            }
            "float" => {
                // float(value)
                if args.len() > 0 {
                    match &args[0] {
                        Value::String(s) => {
                            if let Ok(f) = s.parse::<f64>() {
                                Ok(Value::Decimal(f))
                            } else {
                                Ok(Value::Decimal(0.0))
                            }
                        }
                        Value::Decimal(f) => Ok(Value::Decimal(*f)),
                        Value::Integer(i) => Ok(Value::Decimal(*i as f64)),
                        Value::Bool(b) => Ok(Value::Decimal(if *b { 1.0 } else { 0.0 })),
                        _ => Ok(Value::Decimal(0.0)),
                    }
                } else {
                    Ok(Value::Decimal(0.0))
                }
            }
            "bool" => {
                // bool(value)
                if args.len() > 0 { Ok(Value::Bool(args[0].is_truthy())) } else { Ok(Value::Bool(false)) }
            }
            _ => Err(DejavuError::RuntimeError(RuntimeError::NotImplemented(format!("function {} not implemented", name)))),
        }
    }

    /// Get translation provider
    pub fn translation_provider(&self) -> Option<&dyn TranslationProvider> {
        self.translation_provider.as_deref()
    }

    /// Set translation provider
    pub fn set_translation_provider(&mut self, provider: Box<dyn TranslationProvider>) {
        self.translation_provider = Some(provider);
    }

    /// Translate text (convenience method)
    pub fn translate(&self, key: &str, args: Option<&HashMap<String, String>>) -> DejavuResult<String> {
        if let Some(provider) = self.translation_provider() {
            provider.translate(key, args, provider.get_locale())
        } else {
            Ok(key.to_string())
        }
    }

    /// Register a custom filter
    pub fn register_filter(&mut self, name: &str, filter: std::sync::Arc<dyn filter::Filter>) {
        self.filter_registry.register(name, filter);
    }

    /// Apply a filter to a value
    pub fn apply_filter(&self, name: &str, value: Value, args: &[Value]) -> DejavuResult<Value> {
        self.filter_registry.apply_filter(name, value, args, self)
    }

    /// Get the type environment
    pub fn type_env(&self) -> &crate::types::type_info::TypeEnv {
        &self.type_env
    }

    /// Set the type environment
    pub fn set_type_env(&mut self, type_env: crate::types::type_info::TypeEnv) {
        self.type_env = type_env;
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
