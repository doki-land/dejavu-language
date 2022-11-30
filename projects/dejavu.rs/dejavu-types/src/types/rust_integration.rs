//! Rust type system integration for Dejavu templates

use super::type_info::DejavuType;
use std::{
    any::TypeId,
    collections::{BTreeMap, HashMap},
};

/// Rust type to Dejavu type mapping
pub trait RustTypeToDejavu {
    /// Convert Rust type to Dejavu type
    fn to_dejavu_type() -> DejavuType;
}

/// Implementations for common Rust types
impl RustTypeToDejavu for String {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::String
    }
}

impl RustTypeToDejavu for &str {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::String
    }
}

impl RustTypeToDejavu for i8 {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::Integer
    }
}

impl RustTypeToDejavu for i16 {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::Integer
    }
}

impl RustTypeToDejavu for i32 {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::Integer
    }
}

impl RustTypeToDejavu for i64 {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::Integer
    }
}

impl RustTypeToDejavu for u8 {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::Integer
    }
}

impl RustTypeToDejavu for u16 {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::Integer
    }
}

impl RustTypeToDejavu for u32 {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::Integer
    }
}

impl RustTypeToDejavu for u64 {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::Integer
    }
}

impl RustTypeToDejavu for f32 {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::Decimal
    }
}

impl RustTypeToDejavu for f64 {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::Decimal
    }
}

impl RustTypeToDejavu for bool {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::Bool
    }
}

impl<T: RustTypeToDejavu> RustTypeToDejavu for Vec<T> {
    fn to_dejavu_type() -> DejavuType {
        DejavuType::Array(Box::new(T::to_dejavu_type()))
    }
}

impl<K: RustTypeToDejavu, V: RustTypeToDejavu> RustTypeToDejavu for HashMap<K, V> {
    fn to_dejavu_type() -> DejavuType {
        let mut props = std::collections::HashMap::new();
        // Note: This is a simplified implementation
        // In practice, we would need to analyze the actual keys
        DejavuType::Object(props)
    }
}

impl<T: RustTypeToDejavu> RustTypeToDejavu for Option<T> {
    fn to_dejavu_type() -> DejavuType {
        T::to_dejavu_type()
    }
}

impl<T: RustTypeToDejavu, E: RustTypeToDejavu> RustTypeToDejavu for Result<T, E> {
    fn to_dejavu_type() -> DejavuType {
        T::to_dejavu_type()
    }
}

/// Type mapping registry
pub struct TypeMappingRegistry {
    rust_to_dejavu: HashMap<TypeId, DejavuType>,
    dejavu_to_rust: Vec<(DejavuType, TypeId)>,
}

impl TypeMappingRegistry {
    /// Create a new type mapping registry
    pub fn new() -> Self {
        Self { rust_to_dejavu: HashMap::new(), dejavu_to_rust: Vec::new() }
    }

    /// Register a type mapping
    pub fn register<T: RustTypeToDejavu + 'static>(&mut self) {
        let rust_type_id = TypeId::of::<T>();
        let dejavu_type = T::to_dejavu_type();
        self.rust_to_dejavu.insert(rust_type_id, dejavu_type.clone());
        self.dejavu_to_rust.push((dejavu_type, rust_type_id));
    }

    /// Get Dejavu type from Rust type
    pub fn get_dejavu_type<T: 'static>(&self) -> Option<&DejavuType> {
        let rust_type_id = TypeId::of::<T>();
        self.rust_to_dejavu.get(&rust_type_id)
    }

    /// Get Rust type ID from Dejavu type
    pub fn get_rust_type_id(&self, dejavu_type: &DejavuType) -> Option<&TypeId> {
        self.dejavu_to_rust.iter().find(|(dt, _)| dt == dejavu_type).map(|(_, tid)| tid)
    }
}

impl Default for TypeMappingRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        // Register common types
        registry.register::<String>();
        registry.register::<&str>();
        registry.register::<i32>();
        registry.register::<i64>();
        registry.register::<u32>();
        registry.register::<u64>();
        registry.register::<f32>();
        registry.register::<f64>();
        registry.register::<bool>();
        registry
    }
}

/// Type information for Rust structs
pub struct RustStructTypeInfo {
    pub name: String,
    pub fields: std::collections::HashMap<String, DejavuType>,
}

impl RustStructTypeInfo {
    /// Create a new Rust struct type info
    pub fn new(name: String, fields: std::collections::HashMap<String, DejavuType>) -> Self {
        Self { name, fields }
    }

    /// Convert to Dejavu type
    pub fn to_dejavu_type(&self) -> DejavuType {
        DejavuType::Object(self.fields.clone())
    }
}

/// Trait for types that provide type information
pub trait TypeInfoProvider {
    /// Get type information
    fn get_type_info() -> RustStructTypeInfo;
}
