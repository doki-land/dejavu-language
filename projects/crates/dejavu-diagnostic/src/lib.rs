//! Dejavu 结构化诊断合同。
//!
//! 这个 crate 定义跨宿主、跨 CLI、跨 Doki/Iris 的稳定诊断 wire。
//! 它是 **纯数据合同**，不依赖 `miette`、`thiserror` 或任何宿主适配层。
//!
//! 错误路径：
//!
//! ```text
//! Dejavu 核心（language / IR / engine / config）
//!   → 产出 Diagnostic（code + args + span + severity + stage）
//!   → 宿主适配器消费
//!       ├─ Rust CLI → miette::Report（adapter 在 dejavu 或独立 crate）
//!       ├─ Node CLI → @vmz/diagnostic + @vmz/commander
//!       ├─ Axum    → JSON error response
//!       └─ Doki    → 构建报告聚合
//! ```
//!
//! # 不做什么
//!
//! * **不**提供语言包（locale catalog）—— 宿主 `@vmz/diagnostic` 负责。
//! * **不**提供 source snippet 高亮 —— `miette` / `@vmz/diagnostic` 负责。
//! * **不**做 CLI chrome（help / exit code / 颜色）—— `@vmz/commander` / clap 负责。
//!
//! # 约定
//!
//! * `span` 使用 UTF-8 byte offset，展示时再转 line/column。
//! * `code` 是稳定机器身份字符串，例如 `dejavu.template.circular-extends`。
//! * `args` 是结构化参数对象，不是已经拼好的自然语言句子。
//! * `Display` 只作为无 catalog 时的最后 fallback；真正的展示必须通过宿主 diagnostic formatter。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 诊断严重级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// 阻止继续处理。
    Error,
    /// 不阻止，但需要用户注意。
    Warning,
    /// 纯信息。
    Info,
    /// 内部调试日志，默认不展示给终端用户。
    Debug,
}

/// 诊断产生阶段。
///
/// 用于跨阶段错误聚合（例如 Doki 的"模板加载错误"和"渲染错误"分栏展示）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    /// 模板源码解析阶段。
    Parse,
    /// 模板加载 / 路径解析（extends / include / theme）。
    Load,
    /// 模板语义验证（类型、变量、依赖）。
    Resolve,
    /// 动态渲染执行阶段。
    Render,
    /// AOT 编译（Rust macro）阶段。
    Compile,
    /// 配置文件读取 / 解析阶段。
    Config,
    /// 宿主 / 适配器层错误（不在 Dejavu core 内）。
    Host,
}

/// 源码位置（UTF-8 byte offset）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// byte offset（0-indexed）。
    pub start: usize,
    /// byte offset，exclusive。
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// 诊断合同主体。
///
/// 所有字段都是可选 except `severity` + `code`。
/// 这样 IR 层在没有路径信息时也能产出诊断。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    /// 稳定机器身份，例如 `dejavu.template.circular-extends`。
    pub code: String,
    /// 结构化参数，供 formatter 拼接自然语言。
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub args: BTreeMap<String, ArgValue>,
    /// 主 span。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    /// canonical template / source path。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// canonical template id（path-v1），例如 `theme:layout.dejavu`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    pub stage: Stage,
    /// 相关 span（例如 hint 的位置）。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RelatedSpan>,
    /// 结构化 cause chain（不 `to_string()` 压平）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<Box<Diagnostic>>,
}

/// 诊断参数值（结构化，不是字符串）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<ArgValue>),
    Object(BTreeMap<String, ArgValue>),
    Null,
}

impl From<&str> for ArgValue {
    fn from(s: &str) -> Self {
        ArgValue::String(s.to_string())
    }
}

impl From<&String> for ArgValue {
    fn from(s: &String) -> Self {
        ArgValue::String(s.clone())
    }
}

impl From<String> for ArgValue {
    fn from(s: String) -> Self {
        ArgValue::String(s)
    }
}

impl From<bool> for ArgValue {
    fn from(b: bool) -> Self {
        ArgValue::Boolean(b)
    }
}

impl From<i64> for ArgValue {
    fn from(n: i64) -> Self {
        ArgValue::Number(n as f64)
    }
}

impl From<usize> for ArgValue {
    fn from(n: usize) -> Self {
        ArgValue::Number(n as f64)
    }
}

impl From<f64> for ArgValue {
    fn from(n: f64) -> Self {
        ArgValue::Number(n)
    }
}

impl From<Vec<ArgValue>> for ArgValue {
    fn from(v: Vec<ArgValue>) -> Self {
        ArgValue::Array(v)
    }
}

/// 相关 span（hint / note / secondary location）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelatedSpan {
    pub span: Span,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

// ---------------------------------------------------------------------------
// 构造辅助
// ---------------------------------------------------------------------------

impl Diagnostic {
    /// 新建一个 diagnostic，最少字段：severity + code + stage。
    pub fn new(severity: Severity, code: impl Into<String>, stage: Stage) -> Self {
        Self {
            severity,
            code: code.into(),
            args: BTreeMap::new(),
            span: None,
            path: None,
            template_id: None,
            stage,
            related: Vec::new(),
            cause: None,
        }
    }

    /// 链式加一个 arg。
    pub fn arg(mut self, key: impl Into<String>, value: impl Into<ArgValue>) -> Self {
        self.args.insert(key.into(), value.into());
        self
    }

    /// 链式加主 span。
    pub fn span(mut self, start: usize, end: usize) -> Self {
        self.span = Some(Span::new(start, end));
        self
    }

    /// 链式加路径。
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// 链式加 template id。
    pub fn template_id(mut self, id: impl Into<String>) -> Self {
        self.template_id = Some(id.into());
        self
    }

    /// 链式加相关 span。
    pub fn related(mut self, start: usize, end: usize, label: impl Into<String>) -> Self {
        self.related.push(RelatedSpan {
            span: Span::new(start, end),
            label: label.into(),
            path: self.path.clone(),
        });
        self
    }

    /// 链式套一层 cause。
    pub fn cause(mut self, other: Diagnostic) -> Self {
        self.cause = Some(Box::new(other));
        self
    }
}

// ---------------------------------------------------------------------------
// Display —— fallback，不应用作正式展示
// ---------------------------------------------------------------------------

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sv = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Debug => "debug",
        };
        write!(f, "{sv}: {}", self.code)?;
        if let Some(p) = &self.path {
            write!(f, " @ {p}")?;
        }
        if !self.args.is_empty() {
            write!(f, " args={}", serde_json::to_string(&self.args).unwrap_or_default())?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostic {}

// ---------------------------------------------------------------------------
// 常用错误 code 常量
// ---------------------------------------------------------------------------

pub mod codes {
    // Parse
    pub const PARSE_UNEXPECTED_TOKEN: &str = "dejavu.parse.unexpected-token";
    pub const PARSE_UNCLOSED_EXPRESSION: &str = "dejavu.parse.unclosed-expression";
    pub const PARSE_UNCLOSED_BLOCK: &str = "dejavu.parse.unclosed-block";
    pub const PARSE_TRAILING_INPUT: &str = "dejavu.parse.trailing-input";

    // Template load / resolve
    pub const TEMPLATE_NOT_FOUND: &str = "dejavu.template.template-not-found";
    pub const MISSING_LOADER: &str = "dejavu.template.missing-loader";
    pub const CIRCULAR_EXTENDS: &str = "dejavu.template.circular-extends";

    // Render
    pub const UNKNOWN_FILTER: &str = "dejavu.template.unknown-filter";
    pub const INVALID_ITERABLE: &str = "dejavu.template.invalid-iterable";
    pub const UNDEFINED_VARIABLE: &str = "dejavu.template.undefined-variable";
    pub const SUPER_OUTSIDE_BLOCK: &str = "dejavu.template.super-outside-block";
    pub const NOT_RENDERABLE_NODE: &str = "dejavu.template.not-renderable-node";
    pub const EXTENDS_NOT_TOP_LEVEL: &str = "dejavu.template.extends-not-top-level";
    pub const CALLS_NOT_SUPPORTED: &str = "dejavu.template.calls-not-supported";
    pub const UNKNOWN_UNARY: &str = "dejavu.template.unknown-unary";
    pub const INVALID_EXPRESSION_NODE: &str = "dejavu.template.invalid-expression-node";
    pub const CONTEXT_NOT_OBJECT: &str = "dejavu.template.context-not-object";
    pub const UNKNOWN_BINARY: &str = "dejavu.template.unknown-binary";

    // Config
    pub const CONFIG_READ_FAILED: &str = "dejavu.config.read-failed";
    pub const CONFIG_INVALID_DOCUMENT: &str = "dejavu.config.invalid-document";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_minimal_constructs() {
        let d = Diagnostic::new(Severity::Error, codes::UNKNOWN_FILTER, Stage::Render)
            .arg("filter", "myfilter")
            .path("templates/page.dejavu")
            .span(42, 55);

        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.code, "dejavu.template.unknown-filter");
        assert_eq!(d.args.get("filter").unwrap(), &ArgValue::String("myfilter".into()));
        assert_eq!(d.span, Some(Span::new(42, 55)));
    }

    #[test]
    fn diagnostic_roundtrip_json() {
        let d = Diagnostic::new(Severity::Error, codes::CIRCULAR_EXTENDS, Stage::Load)
            .arg("chain", vec![ArgValue::String("a.dejavu".into()), ArgValue::String("b.dejavu".into())])
            .template_id("theme:layout.dejavu")
            .cause(Diagnostic::new(Severity::Error, "io.failed", Stage::Host));

        let json = serde_json::to_string_pretty(&d).unwrap();
        let roundtripped: Diagnostic = serde_json::from_str(&json).unwrap();
        assert_eq!(d, roundtripped);
    }

    #[test]
    fn related_spans_serialize() {
        let d = Diagnostic::new(Severity::Warning, codes::PARSE_UNEXPECTED_TOKEN, Stage::Parse)
            .span(10, 15)
            .related(20, 30, "expected here");
        let json = serde_json::to_string(&d).unwrap();
        assert!(json.contains("related"));
        assert!(json.contains("expected here"));
    }

    #[test]
    fn empty_args_not_serialized() {
        let d = Diagnostic::new(Severity::Info, "test", Stage::Config);
        let json = serde_json::to_string(&d).unwrap();
        assert!(!json.contains("args"));
    }
}
