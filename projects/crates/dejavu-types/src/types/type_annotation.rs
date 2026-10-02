//! Type annotation support for Dejavu templates

use super::type_info::{DejavuType, TypeEnv};
use crate::{DejavuError, DejavuResult, RuntimeError};

/// Type annotation parser
pub struct TypeAnnotationParser {
    type_env: TypeEnv,
}

impl TypeAnnotationParser {
    /// Create a new type annotation parser
    pub fn new() -> Self {
        Self { type_env: TypeEnv::new() }
    }

    /// Parse a type annotation string into a DejavuType
    pub fn parse_type_annotation(&self, annotation: &str) -> DejavuResult<DejavuType> {
        let annotation = annotation.trim();

        match annotation {
            "string" => Ok(DejavuType::String),
            "integer" | "int" => Ok(DejavuType::Integer),
            "decimal" | "float" | "number" => Ok(DejavuType::Decimal),
            "boolean" | "bool" => Ok(DejavuType::Bool),
            "null" => Ok(DejavuType::Null),
            "any" => Ok(DejavuType::Any),
            _ => {
                // Handle array types: array<string>
                if annotation.starts_with("array<") && annotation.ends_with(">") {
                    let inner_type = &annotation[6..annotation.len() - 1];
                    let parsed_inner = self.parse_type_annotation(inner_type)?;
                    Ok(DejavuType::Array(Box::new(parsed_inner)))
                }
                // Handle object types: object<{name: string, age: integer}>
                else if annotation.starts_with("object<{") && annotation.ends_with("}>") {
                    let props_str = &annotation[8..annotation.len() - 2];
                    let props = self.parse_object_properties(props_str)?;
                    Ok(DejavuType::Object(props))
                }
                // Handle function types: function<(string, integer) -> string>
                else if annotation.starts_with("function<(") && annotation.contains(") -> ") {
                    let parts: Vec<&str> = annotation[9..annotation.len() - 1].split(") -> ").collect();
                    if parts.len() == 2 {
                        let args_str = parts[0];
                        let return_str = parts[1];

                        let args = self.parse_function_args(args_str)?;
                        let return_type = self.parse_type_annotation(return_str)?;

                        Ok(DejavuType::Function(args, Box::new(return_type)))
                    } else {
                        Err(DejavuError::RuntimeError(RuntimeError::NotImplemented(format!(
                            "Invalid function type annotation: {}",
                            annotation
                        ))))
                    }
                } else {
                    // Treat as custom type (object by default)
                    Ok(DejavuType::Object(std::collections::HashMap::new()))
                }
            }
        }
    }

    /// Parse object properties from a string
    fn parse_object_properties(&self, props_str: &str) -> DejavuResult<std::collections::HashMap<String, DejavuType>> {
        let mut props = std::collections::HashMap::new();

        for prop in props_str.split(",").map(|p| p.trim()) {
            if prop.is_empty() {
                continue;
            }

            let parts: Vec<&str> = prop.split(":").map(|p| p.trim()).collect();
            if parts.len() != 2 {
                return Err(DejavuError::RuntimeError(RuntimeError::NotImplemented(format!(
                    "Invalid object property: {}",
                    prop
                ))));
            }

            let name = parts[0].trim_matches('"').trim_matches('\'');
            let type_str = parts[1];

            let ty = self.parse_type_annotation(type_str)?;
            props.insert(name.to_string(), ty);
        }

        Ok(props)
    }

    /// Parse function arguments from a string
    fn parse_function_args(&self, args_str: &str) -> DejavuResult<Vec<DejavuType>> {
        let mut args = Vec::new();

        for arg in args_str.split(",").map(|a| a.trim()) {
            if arg.is_empty() {
                continue;
            }

            let ty = self.parse_type_annotation(arg)?;
            args.push(ty);
        }

        Ok(args)
    }

    /// Get the type environment
    pub fn type_env(&self) -> &TypeEnv {
        &self.type_env
    }

    /// Set the type environment
    pub fn set_type_env(&mut self, type_env: TypeEnv) {
        self.type_env = type_env;
    }
}

impl Default for TypeAnnotationParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Type annotation utilities
pub mod utils {
    use super::*;

    /// Extract type annotation from variable declaration
    pub fn extract_type_annotation(declaration: &str) -> Option<(String, String)> {
        // Look for pattern like "let x: type = value"
        let parts: Vec<&str> = declaration.split('=').collect();
        if parts.len() >= 2 {
            let var_part = parts[0].trim();
            if var_part.starts_with("let ") {
                let var_def = &var_part[4..];
                if let Some(colon_idx) = var_def.find(':') {
                    let var_name = var_def[..colon_idx].trim();
                    let type_annotation = var_def[colon_idx + 1..].trim();
                    return Some((var_name.to_string(), type_annotation.to_string()));
                }
            }
        }
        None
    }

    /// Parse type annotations in template content
    pub fn parse_type_annotations(content: &str) -> std::collections::HashMap<String, DejavuType> {
        let parser = TypeAnnotationParser::new();
        let mut type_mappings = std::collections::HashMap::new();

        for line in content.lines() {
            if let Some((var_name, type_annotation)) = extract_type_annotation(line) {
                if let Ok(ty) = parser.parse_type_annotation(&type_annotation) {
                    type_mappings.insert(var_name, ty);
                }
            }
        }

        type_mappings
    }
}
