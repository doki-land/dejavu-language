//! Error types for Dejavu template engine
//!
//! This module defines all error types used by the Dejavu template engine,
//! including parse errors, compilation errors, and runtime errors.

use std::fmt;

/// Result type for Dejavu operations
///
/// Alias for `Result<T, DejavuError>`.
pub type DejavuResult<T> = Result<T, DejavuError>;

/// Main error type for Dejavu template engine
///
/// This enum represents all possible errors that can occur during
/// template parsing, compilation, or execution.
#[derive(Debug, Clone)]
pub enum DejavuError {
    /// Error occurred during template parsing
    ParseError(String),
    /// Error occurred during AOT compilation
    CompileError(CompileError),
    /// Error occurred during dynamic template execution
    RuntimeError(RuntimeError),
    /// IO error (e.g., file reading/writing failed)
    IoError(String),
    /// Template not found in the registry
    TemplateNotFound(String),
    /// Invalid template configuration
    InvalidConfig(String),
}

/// Compilation errors for AOT mode
///
/// Errors that can occur when compiling templates to target language code.
#[derive(Debug, Clone)]
pub enum CompileError {
    /// The specified target language is not supported
    UnsupportedTarget(String),
    /// Error occurred during code generation
    CodeGenError {
        /// 错误消息
        message: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
    },
    /// Type checking failed
    TypeCheckError(TypeCheckError),
    /// Template declaration is missing (required for AOT)
    MissingTemplateDeclaration {
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
    },
    /// The template path is invalid
    InvalidTemplatePath(String),
    /// Undefined variable
    UndefinedVariable {
        /// 未定义变量名
        name: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
    },
    /// Invalid expression
    InvalidExpression {
        /// 错误消息
        message: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
    },
    /// Mismatched block tags
    MismatchedBlock {
        /// 期望的块标签
        expected: String,
        /// 实际找到的块标签
        found: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
    },
    /// Missing translation key
    MissingTranslationKey {
        /// 缺失的翻译键
        key: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
    },
    /// Untranslated string found
    UntranslatedString {
        /// 未翻译的原始字符串
        string: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
    },
    /// Missing required parameter for translation
    MissingRequiredParameter {
        /// 缺失的必填参数名
        parameter: String,
        /// 对应的翻译键
        key: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
    },
    /// Unknown parameter for translation
    UnknownParameter {
        /// 未知的参数名
        parameter: String,
        /// 对应的翻译键
        key: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
    },
}

/// Type checking errors
///
/// Detailed errors that can occur during type checking of Dejavu templates.
#[derive(Debug, Clone)]
pub enum TypeCheckError {
    /// Type mismatch between expected and actual type
    TypeMismatch {
        /// 期望的类型
        expected: String,
        /// 实际找到的类型
        found: String,
        /// 附加错误消息
        message: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
        /// 修复建议
        suggestion: Option<String>,
    },
    /// Undefined variable
    UndefinedVariable {
        /// 未定义变量名
        name: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
        /// 修复建议
        suggestion: Option<String>,
    },
    /// Undefined function
    UndefinedFunction {
        /// 未定义函数名
        name: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
        /// 修复建议
        suggestion: Option<String>,
    },
    /// Undefined property
    UndefinedProperty {
        /// 所属对象名
        object: String,
        /// 缺失的属性名
        property: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
        /// 修复建议
        suggestion: Option<String>,
    },
    /// Invalid operation for the given type
    InvalidOperation {
        /// 操作名称
        operation: String,
        /// 操作数类型
        operand_type: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
        /// 修复建议
        suggestion: Option<String>,
    },
    /// Function argument count mismatch
    ArgumentCountMismatch {
        /// 函数名
        function: String,
        /// 期望的参数数量
        expected: usize,
        /// 实际传入的参数数量
        got: usize,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
        /// 修复建议
        suggestion: Option<String>,
    },
    /// Index out of bounds
    IndexOutOfBounds {
        /// 越界的索引值
        index: usize,
        /// 容器长度
        length: usize,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
        /// 修复建议
        suggestion: Option<String>,
    },
    /// Division by zero
    DivisionByZero {
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
        /// 修复建议
        suggestion: Option<String>,
    },
    /// Invalid type annotation
    InvalidTypeAnnotation {
        /// 无效的类型注解字符串
        annotation: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
        /// 修复建议
        suggestion: Option<String>,
    },
    /// Type inference failed
    TypeInferenceFailed {
        /// 无法推断类型的表达式
        expression: String,
        /// 发生行号（1-based），无则为 None
        line: Option<usize>,
        /// 发生列号（1-based），无则为 None
        column: Option<usize>,
        /// 修复建议
        suggestion: Option<String>,
    },
}

/// Runtime errors for Dyn mode
///
/// Errors that can occur during dynamic template execution.
#[derive(Debug, Clone)]
pub enum RuntimeError {
    /// Variable not found in the current context
    VariableNotFound(String),
    /// Function not found in the current context
    FunctionNotFound(String),
    /// Filter not found in the filter registry
    FilterNotFound(String),
    /// Property not found on an object
    PropertyNotFound(String),
    /// Type mismatch between expected and actual value
    TypeMismatch {
        /// 期望的类型
        expected: String,
        /// 实际值的类型
        found: String,
    },
    /// Division by zero attempted
    DivisionByZero,
    /// Array or string index out of bounds
    IndexOutOfBounds {
        /// 越界的索引值
        index: usize,
        /// 容器长度
        length: usize,
    },
    /// Invalid operation for the given value type
    InvalidOperation(String),
    /// Stack overflow occurred (e.g., deep recursion)
    StackOverflow,
    /// Infinite loop detected
    InfiniteLoop,
    /// Loop iteration limit exceeded
    LoopLimitExceeded,
    /// Recursion depth limit exceeded
    RecursionLimitExceeded,
    /// Feature is not yet implemented
    NotImplemented(String),
    /// Macro not found in the macro registry
    MacroNotFound {
        /// 缺失的宏名称
        name: String,
    },
    /// Too many arguments passed to a macro
    TooManyArguments {
        /// 宏名称
        name: String,
        /// 期望的参数数量
        expected: usize,
        /// 实际传入的参数数量
        got: usize,
    },
    /// Duplicate keyword argument provided to a macro
    DuplicateKeywordArgument {
        /// 重复的关键字参数名
        name: String,
    },
    /// Unknown keyword argument provided to a macro
    UnknownKeywordArgument {
        /// 未知的关键字参数名
        name: String,
    },
    /// Required argument missing for a macro
    MissingRequiredArgument {
        /// 缺失的必填参数名
        name: String,
    },
    /// Circular template reference detected
    CircularTemplateReference {
        /// 被检测为循环引用的模板名称
        template: String,
        /// 引用链上所有涉及的模板名
        chain: Vec<String>,
    },
}

impl fmt::Display for DejavuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DejavuError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            DejavuError::CompileError(err) => write!(f, "Compile error: {}", err),
            DejavuError::RuntimeError(err) => write!(f, "Runtime error: {}", err),
            DejavuError::IoError(msg) => write!(f, "IO error: {}", msg),
            DejavuError::TemplateNotFound(name) => write!(f, "Template not found: {}", name),
            DejavuError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::UnsupportedTarget(target) => {
                write!(f, "❌ 编译错误: 不支持的目标语言: {}", target)
            }
            CompileError::CodeGenError { message, line, column } => {
                if let (Some(line), Some(column)) = (line, column) {
                    write!(f, "❌ 编译错误: 代码生成失败 (第{}行, 第{}列): {}", line, column, message)
                } else {
                    write!(f, "❌ 编译错误: 代码生成失败: {}", message)
                }
            }
            CompileError::TypeCheckError(type_error) => {
                write!(f, "❌ 编译错误: {}", type_error)
            }
            CompileError::MissingTemplateDeclaration { line, column } => {
                if let (Some(line), Some(column)) = (line, column) {
                    write!(f, "❌ 编译错误: 缺少模板声明 (第{}行, 第{}列)", line, column)
                } else {
                    write!(f, "❌ 编译错误: 缺少模板声明")
                }
            }
            CompileError::InvalidTemplatePath(path) => {
                write!(f, "❌ 编译错误: 无效的模板路径: {}", path)
            }
            CompileError::UndefinedVariable { name, line, column } => {
                if let (Some(line), Some(column)) = (line, column) {
                    write!(f, "❌ 编译错误: 未定义的变量 '{}' (第{}行, 第{}列)", name, line, column)
                } else {
                    write!(f, "❌ 编译错误: 未定义的变量: {}", name)
                }
            }
            CompileError::InvalidExpression { message, line, column } => {
                if let (Some(line), Some(column)) = (line, column) {
                    write!(f, "❌ 编译错误: 无效的表达式 (第{}行, 第{}列): {}", line, column, message)
                } else {
                    write!(f, "❌ 编译错误: 无效的表达式: {}", message)
                }
            }
            CompileError::MismatchedBlock { expected, found, line, column } => {
                if let (Some(line), Some(column)) = (line, column) {
                    write!(f, "❌ 编译错误: 块标签不匹配 (第{}行, 第{}列): 期望 '{}', 找到 '{}'", line, column, expected, found)
                } else {
                    write!(f, "❌ 编译错误: 块标签不匹配: 期望 '{}', 找到 '{}'", expected, found)
                }
            }
            CompileError::MissingTranslationKey { key, line, column } => {
                if let (Some(line), Some(column)) = (line, column) {
                    write!(f, "❌ 编译错误: 缺少翻译键 '{}' (第{}行, 第{}列)", key, line, column)
                } else {
                    write!(f, "❌ 编译错误: 缺少翻译键: {}", key)
                }
            }
            CompileError::UntranslatedString { string, line, column } => {
                if let (Some(line), Some(column)) = (line, column) {
                    write!(f, "❌ 编译错误: 未翻译的字符串 '{}' (第{}行, 第{}列)", string, line, column)
                } else {
                    write!(f, "❌ 编译错误: 未翻译的字符串: {}", string)
                }
            }
            CompileError::MissingRequiredParameter { parameter, key, line, column } => {
                if let (Some(line), Some(column)) = (line, column) {
                    write!(f, "❌ 编译错误: 翻译键 '{}' 缺少必填参数 '{}' (第{}行, 第{}列)", key, parameter, line, column)
                } else {
                    write!(f, "❌ 编译错误: 翻译键 '{}' 缺少必填参数 '{}'", key, parameter)
                }
            }
            CompileError::UnknownParameter { parameter, key, line, column } => {
                if let (Some(line), Some(column)) = (line, column) {
                    write!(f, "❌ 编译错误: 翻译键 '{}' 存在未知参数 '{}' (第{}行, 第{}列)", key, parameter, line, column)
                } else {
                    write!(f, "❌ 编译错误: 翻译键 '{}' 存在未知参数 '{}'", key, parameter)
                }
            }
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::VariableNotFound(name) => write!(f, "Variable not found: {}", name),
            RuntimeError::FunctionNotFound(name) => write!(f, "Function not found: {}", name),
            RuntimeError::FilterNotFound(name) => write!(f, "Filter not found: {}", name),
            RuntimeError::PropertyNotFound(name) => write!(f, "Property not found: {}", name),
            RuntimeError::TypeMismatch { expected, found } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::IndexOutOfBounds { index, length } => {
                write!(f, "Index {} out of bounds for length {}", index, length)
            }
            RuntimeError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            RuntimeError::StackOverflow => write!(f, "Stack overflow"),
            RuntimeError::InfiniteLoop => write!(f, "Infinite loop detected"),
            RuntimeError::LoopLimitExceeded => write!(f, "Loop iteration limit exceeded"),
            RuntimeError::RecursionLimitExceeded => write!(f, "Recursion limit exceeded"),
            RuntimeError::NotImplemented(feature) => write!(f, "Not implemented: {}", feature),
            RuntimeError::MacroNotFound { name } => write!(f, "Macro not found: {}", name),
            RuntimeError::TooManyArguments { name, expected, got } => {
                write!(f, "Too many arguments for macro {}: expected {}, got {}", name, expected, got)
            }
            RuntimeError::DuplicateKeywordArgument { name } => {
                write!(f, "Duplicate keyword argument: {}", name)
            }
            RuntimeError::UnknownKeywordArgument { name } => {
                write!(f, "Unknown keyword argument: {}", name)
            }
            RuntimeError::MissingRequiredArgument { name } => {
                write!(f, "Missing required argument: {}", name)
            }
            RuntimeError::CircularTemplateReference { template, chain } => {
                let chain_str = chain.join(" -> ");
                write!(f, "Circular template reference detected: {} in chain {}", template, chain_str)
            }
        }
    }
}

impl fmt::Display for TypeCheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeCheckError::TypeMismatch { expected, found, message, line, column, suggestion } => {
                let location = if let (Some(line), Some(column)) = (line, column) {
                    format!(" (第{}行, 第{}列)", line, column)
                } else {
                    String::new()
                };
                let mut error_msg = format!("类型不匹配{location}: 期望 {}, 找到 {}", expected, found);
                if !message.is_empty() {
                    error_msg.push_str(&format!(" - {}", message));
                }
                if let Some(suggestion) = suggestion {
                    error_msg.push_str(&format!("\n💡 建议: {}", suggestion));
                }
                write!(f, "{}", error_msg)
            }
            TypeCheckError::UndefinedVariable { name, line, column, suggestion } => {
                let location = if let (Some(line), Some(column)) = (line, column) {
                    format!(" (第{}行, 第{}列)", line, column)
                } else {
                    String::new()
                };
                let mut error_msg = format!("未定义的变量 '{}'{}", name, location);
                if let Some(suggestion) = suggestion {
                    error_msg.push_str(&format!("\n💡 建议: {}", suggestion));
                }
                write!(f, "{}", error_msg)
            }
            TypeCheckError::UndefinedFunction { name, line, column, suggestion } => {
                let location = if let (Some(line), Some(column)) = (line, column) {
                    format!(" (第{}行, 第{}列)", line, column)
                } else {
                    String::new()
                };
                let mut error_msg = format!("未定义的函数 '{}'{}", name, location);
                if let Some(suggestion) = suggestion {
                    error_msg.push_str(&format!("\n💡 建议: {}", suggestion));
                }
                write!(f, "{}", error_msg)
            }
            TypeCheckError::UndefinedProperty { object, property, line, column, suggestion } => {
                let location = if let (Some(line), Some(column)) = (line, column) {
                    format!(" (第{}行, 第{}列)", line, column)
                } else {
                    String::new()
                };
                let mut error_msg = format!("对象 '{}' 上不存在属性 '{}'{}", object, property, location);
                if let Some(suggestion) = suggestion {
                    error_msg.push_str(&format!("\n💡 建议: {}", suggestion));
                }
                write!(f, "{}", error_msg)
            }
            TypeCheckError::InvalidOperation { operation, operand_type, line, column, suggestion } => {
                let location = if let (Some(line), Some(column)) = (line, column) {
                    format!(" (第{}行, 第{}列)", line, column)
                } else {
                    String::new()
                };
                let mut error_msg = format!("对类型 '{}' 执行操作 '{}' 无效{}", operand_type, operation, location);
                if let Some(suggestion) = suggestion {
                    error_msg.push_str(&format!("\n💡 建议: {}", suggestion));
                }
                write!(f, "{}", error_msg)
            }
            TypeCheckError::ArgumentCountMismatch { function, expected, got, line, column, suggestion } => {
                let location = if let (Some(line), Some(column)) = (line, column) {
                    format!(" (第{}行, 第{}列)", line, column)
                } else {
                    String::new()
                };
                let mut error_msg = format!("函数 '{}' 参数数量不匹配{location}: 期望 {}, 得到 {}", function, expected, got);
                if let Some(suggestion) = suggestion {
                    error_msg.push_str(&format!("\n💡 建议: {}", suggestion));
                }
                write!(f, "{}", error_msg)
            }
            TypeCheckError::IndexOutOfBounds { index, length, line, column, suggestion } => {
                let location = if let (Some(line), Some(column)) = (line, column) {
                    format!(" (第{}行, 第{}列)", line, column)
                } else {
                    String::new()
                };
                let mut error_msg = format!("索引越界{location}: 索引 {} 超出长度 {}", index, length);
                if let Some(suggestion) = suggestion {
                    error_msg.push_str(&format!("\n💡 建议: {}", suggestion));
                }
                write!(f, "{}", error_msg)
            }
            TypeCheckError::DivisionByZero { line, column, suggestion } => {
                let location = if let (Some(line), Some(column)) = (line, column) {
                    format!(" (第{}行, 第{}列)", line, column)
                } else {
                    String::new()
                };
                let mut error_msg = format!("除以零错误{}", location);
                if let Some(suggestion) = suggestion {
                    error_msg.push_str(&format!("\n💡 建议: {}", suggestion));
                }
                write!(f, "{}", error_msg)
            }
            TypeCheckError::InvalidTypeAnnotation { annotation, line, column, suggestion } => {
                let location = if let (Some(line), Some(column)) = (line, column) {
                    format!(" (第{}行, 第{}列)", line, column)
                } else {
                    String::new()
                };
                let mut error_msg = format!("无效的类型注解 '{}'{}", annotation, location);
                if let Some(suggestion) = suggestion {
                    error_msg.push_str(&format!("\n💡 建议: {}", suggestion));
                }
                write!(f, "{}", error_msg)
            }
            TypeCheckError::TypeInferenceFailed { expression, line, column, suggestion } => {
                let location = if let (Some(line), Some(column)) = (line, column) {
                    format!(" (第{}行, 第{}列)", line, column)
                } else {
                    String::new()
                };
                let mut error_msg = format!("类型推断失败{location}: 无法推断表达式 '{}' 的类型", expression);
                if let Some(suggestion) = suggestion {
                    error_msg.push_str(&format!("\n💡 建议: {}", suggestion));
                }
                write!(f, "{}", error_msg)
            }
        }
    }
}

impl std::error::Error for DejavuError {}
impl std::error::Error for CompileError {}
impl std::error::Error for RuntimeError {}
impl std::error::Error for TypeCheckError {}

impl From<std::io::Error> for DejavuError {
    fn from(err: std::io::Error) -> Self {
        DejavuError::IoError(err.to_string())
    }
}

impl From<CompileError> for DejavuError {
    fn from(err: CompileError) -> Self {
        DejavuError::CompileError(err)
    }
}

impl From<RuntimeError> for DejavuError {
    fn from(err: RuntimeError) -> Self {
        DejavuError::RuntimeError(err)
    }
}
