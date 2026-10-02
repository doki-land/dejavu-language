//! Type information for Dejavu templates

/// Type representation for Dejavu templates
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DejavuType {
    /// String type
    String,
    /// Integer type
    Integer,
    /// Decimal (floating-point) type
    Decimal,
    /// Boolean type
    Bool,
    /// Array type with element type
    Array(Box<DejavuType>),
    /// Object type with property types
    Object(std::collections::HashMap<String, DejavuType>),
    /// Function type with argument types and return type
    Function(Vec<DejavuType>, Box<DejavuType>),
    /// Null type
    Null,
    /// Any type (fallback)
    Any,
}

impl DejavuType {
    /// Check if this type is a subtype of another type
    pub fn is_subtype_of(&self, other: &DejavuType) -> bool {
        match (self, other) {
            (_, DejavuType::Any) => true,
            (DejavuType::Null, _) => true,
            (DejavuType::String, DejavuType::String) => true,
            (DejavuType::Integer, DejavuType::Integer) => true,
            (DejavuType::Decimal, DejavuType::Decimal) => true,
            (DejavuType::Bool, DejavuType::Bool) => true,
            (DejavuType::Array(inner), DejavuType::Array(other_inner)) => inner.is_subtype_of(other_inner),
            (DejavuType::Object(props), DejavuType::Object(other_props)) => {
                props.iter().all(|(name, ty)| other_props.get(name).map(|other_ty| ty.is_subtype_of(other_ty)).unwrap_or(false))
            }
            (DejavuType::Function(args, ret), DejavuType::Function(other_args, other_ret)) => {
                if args.len() != other_args.len() {
                    return false;
                }
                args.iter().zip(other_args.iter()).all(|(arg, other_arg)| arg.is_subtype_of(other_arg))
                    && ret.is_subtype_of(other_ret)
            }
            _ => false,
        }
    }

    /// Get the type name as string
    pub fn name(&self) -> String {
        match self {
            DejavuType::String => "string".to_string(),
            DejavuType::Integer => "integer".to_string(),
            DejavuType::Decimal => "decimal".to_string(),
            DejavuType::Bool => "boolean".to_string(),
            DejavuType::Array(inner) => format!("array<{}>", inner.name()),
            DejavuType::Object(_) => "object".to_string(),
            DejavuType::Function(_, ret) => format!("function -> {}", ret.name()),
            DejavuType::Null => "null".to_string(),
            DejavuType::Any => "any".to_string(),
        }
    }
}

/// Type environment for type checking
#[derive(Debug, Clone)]
pub struct TypeEnv {
    /// Variable types
    variables: std::collections::HashMap<String, DejavuType>,
    /// Function types
    functions: std::collections::HashMap<String, DejavuType>,
}

impl TypeEnv {
    /// Create a new type environment
    pub fn new() -> Self {
        Self { variables: std::collections::HashMap::new(), functions: std::collections::HashMap::new() }
    }

    /// Add a variable type
    pub fn add_variable(&mut self, name: String, ty: DejavuType) {
        self.variables.insert(name, ty);
    }

    /// Get a variable type
    pub fn get_variable(&self, name: &str) -> Option<&DejavuType> {
        self.variables.get(name)
    }

    /// Add a function type
    pub fn add_function(&mut self, name: String, ty: DejavuType) {
        self.functions.insert(name, ty);
    }

    /// Get a function type
    pub fn get_function(&self, name: &str) -> Option<&DejavuType> {
        self.functions.get(name)
    }

    /// Extend the environment with another environment
    pub fn extend(&mut self, other: &TypeEnv) {
        self.variables.extend(other.variables.clone());
        self.functions.extend(other.functions.clone());
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

use std::hash::{Hash, Hasher};

impl Hash for DejavuType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DejavuType::String => 0.hash(state),
            DejavuType::Integer => 1.hash(state),
            DejavuType::Decimal => 2.hash(state),
            DejavuType::Bool => 3.hash(state),
            DejavuType::Array(inner) => {
                4.hash(state);
                inner.hash(state);
            }
            DejavuType::Object(props) => {
                5.hash(state);
                // 对 HashMap 进行排序后哈希，确保顺序不影响哈希结果
                let mut sorted: Vec<_> = props.iter().collect();
                sorted.sort_by(|a, b| a.0.cmp(b.0));
                for (k, v) in sorted {
                    k.hash(state);
                    v.hash(state);
                }
            }
            DejavuType::Function(args, ret) => {
                6.hash(state);
                args.hash(state);
                ret.hash(state);
            }
            DejavuType::Null => 7.hash(state),
            DejavuType::Any => 8.hash(state),
        }
    }
}
