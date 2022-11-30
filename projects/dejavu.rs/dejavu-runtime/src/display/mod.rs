//! Display utilities for escaping content
use alloc::string::{String, ToString};
use core::fmt::{Display, Formatter, Write};

/// Trait for escaping content
pub trait Escaper {
    /// Write escaped content to a writer
    fn write_escaped<W>(&self, fmt: W, string: &str) -> core::fmt::Result
    where
        W: Write;
}

/// HTML 转义配置
#[derive(Debug, Clone, Default)]
pub struct HtmlEscapeConfig {
    /// 是否启用自动 HTML 转义
    pub enabled: bool,
    /// 是否对属性值进行转义
    pub escape_attributes: bool,
    /// 是否对内容进行转义
    pub escape_content: bool,
}

/// A wrapper around a display value that can be escaped
#[derive(Debug)]
pub struct EscapeDisplay<E, T>
where
    E: Escaper,
    T: Display,
{
    value: DisplayValue<T>,
    escaper: E,
    config: HtmlEscapeConfig,
}

/// A writer that escapes content as it writes
#[derive(Debug)]
pub struct EscapeWriter<'a, E, W> {
    fmt: W,
    escaper: &'a E,
    config: &'a HtmlEscapeConfig,
}

/// A wrapper around a string that is escaped when displayed
#[derive(Debug)]
pub struct Escaped<'a, E>
where
    E: Escaper,
{
    string: &'a str,
    escaper: E,
    config: HtmlEscapeConfig,
}

impl<E, T> EscapeDisplay<E, T>
where
    E: Escaper,
    T: Display,
{
    /// Create a new escape display that will escape the value
    pub fn dangerous(value: T, escaper: E) -> Self {
        Self { value: DisplayValue::Unsafe(value), escaper, config: HtmlEscapeConfig::default() }
    }

    /// Create a new escape display that will escape the value with config
    pub fn dangerous_with_config(value: T, escaper: E, config: HtmlEscapeConfig) -> Self {
        Self { value: DisplayValue::Unsafe(value), escaper, config }
    }

    /// Create a new escape display that will not escape the value
    pub fn safe(value: T, escaper: E) -> Self {
        Self { value: DisplayValue::Safe(value), escaper, config: HtmlEscapeConfig::default() }
    }

    /// Create a new escape display that will not escape the value with config
    pub fn safe_with_config(value: T, escaper: E, config: HtmlEscapeConfig) -> Self {
        Self { value: DisplayValue::Safe(value), escaper, config }
    }

    /// Mark the value as safe, preventing escaping
    #[must_use]
    pub fn mark_safe(mut self) -> EscapeDisplay<E, T> {
        self.value = match self.value {
            DisplayValue::Unsafe(t) => DisplayValue::Safe(t),
            _ => self.value,
        };
        self
    }

    /// Set the HTML escape config
    #[must_use]
    pub fn with_config(mut self, config: HtmlEscapeConfig) -> EscapeDisplay<E, T> {
        self.config = config;
        self
    }
}

impl<E, T> Display for EscapeDisplay<E, T>
where
    E: Escaper,
    T: Display,
{
    fn fmt(&self, fmt: &mut Formatter<'_>) -> core::fmt::Result {
        match self.value {
            DisplayValue::Unsafe(ref t) => {
                if self.config.enabled {
                    write!(EscapeWriter { fmt, escaper: &self.escaper, config: &self.config }, "{t}")
                } else {
                    t.fmt(fmt)
                }
            }
            DisplayValue::Safe(ref t) => t.fmt(fmt),
        }
    }
}

impl<E, W> Write for EscapeWriter<'_, E, W>
where
    W: Write,
    E: Escaper,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.config.enabled { self.escaper.write_escaped(&mut self.fmt, s) } else { self.fmt.write_str(s) }
    }
}

/// Escape a string using the given escaper
pub fn escape<E>(string: &str, escaper: E) -> Escaped<'_, E>
where
    E: Escaper,
{
    Escaped { string, escaper, config: HtmlEscapeConfig::default() }
}

/// Escape a string using the given escaper with config
pub fn escape_with_config<E>(string: &str, escaper: E, config: HtmlEscapeConfig) -> Escaped<'_, E>
where
    E: Escaper,
{
    Escaped { string, escaper, config }
}

impl<E> Display for Escaped<'_, E>
where
    E: Escaper,
{
    fn fmt(&self, fmt: &mut Formatter<'_>) -> core::fmt::Result {
        if self.config.enabled { self.escaper.write_escaped(fmt, self.string) } else { fmt.write_str(self.string) }
    }
}

#[derive(Debug, PartialEq)]
enum DisplayValue<T>
where
    T: Display,
{
    Safe(T),
    Unsafe(T),
}

/// 安全地转义 HTML 内容
pub fn safe_html_escape<E>(string: &str, escaper: E, config: HtmlEscapeConfig) -> String
where
    E: Escaper,
{
    if !config.enabled || !config.escape_content {
        return string.to_string();
    }
    let mut result = String::new();
    escaper.write_escaped(&mut result, string).unwrap();
    result
}

/// 安全地转义 HTML 属性值
pub fn safe_html_attribute_escape<E>(string: &str, escaper: E, config: HtmlEscapeConfig) -> String
where
    E: Escaper,
{
    if !config.enabled || !config.escape_attributes {
        return string.to_string();
    }
    let mut result = String::new();
    escaper.write_escaped(&mut result, string).unwrap();
    result
}
