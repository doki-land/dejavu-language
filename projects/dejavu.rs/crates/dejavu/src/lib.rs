//! # dejavu
//!
//! Public Rust surface for the Dejavu template engine.
//!
//! ```ignore
//! use dejavu::Dejavu;
//! let out = Dejavu::render_source("Hello, <% name %>!", &serde_json::json!({"name":"World"}))?;
//! ```
//!
//! Application code should depend on **`dejavu`** only.
//! Advanced crates (`dejavu-language`, `dejavu-ir`, `dejavu-engine`, `dejavu-runtime`)
//! remain available for integrations.

#![warn(missing_docs)]

pub use dejavu_engine::{DejavuEngine, parse, parse_error_diagnostic, render, render_source};
pub use dejavu_ir::{
    ArgValue, Diagnostic, IrDocument, IrError, IrNode, Language, RelatedSpan, Severity, Span, Stage, TemplateConfig,
    codes, diag_load, diag_render, normalize_value,
};
pub use dejavu_language::{ParseError, ParseResult};
pub use dejavu_runtime::{
    EscapeDisplay, Escaper, HtmlEscapeConfig, Template, escape, escape_with_config, safe_html_attribute_escape,
    safe_html_escape,
};
pub use dejavu_runtime::{escaper, looper};
pub use dejavu_types::{Context, Value};

/// Canonical user-facing facade (same role as `Dejavu` in other host languages).
pub struct Dejavu;

impl Dejavu {
    /// Parse source → IR (default `<% %>` delimiters).
    pub fn parse(source: &str) -> ParseResult<IrDocument> {
        parse(source)
    }

    /// Parse source → IR with explicit Language config (custom delimiters).
    pub fn parse_with(source: &str, language: &Language) -> ParseResult<IrDocument> {
        dejavu_language::parse_with(source, "template.dejavu", language)
    }

    /// Render IR + JSON context.
    pub fn render(doc: &IrDocument, ctx: &serde_json::Value) -> Result<String, IrError> {
        render(doc, ctx)
    }

    /// Parse then render (default `<% %>` delimiters).
    pub fn render_source(source: &str, ctx: &serde_json::Value) -> Result<String, IrError> {
        render_source(source, ctx)
    }

    /// Parse then render with explicit Language config (custom delimiters).
    pub fn render_source_with(
        source: &str,
        ctx: &serde_json::Value,
        language: &Language,
    ) -> Result<String, IrError> {
        let doc = Self::parse_with(source, language).map_err(|e| IrError::from(parse_error_diagnostic(e)))?;
        render(&doc, ctx)
    }

    /// Syntax check (default `<% %>` delimiters).
    pub fn check(source: &str) -> ParseResult<()> {
        parse(source).map(|_| ())
    }

    /// Syntax check with explicit Language config.
    pub fn check_with(source: &str, language: &Language) -> ParseResult<()> {
        Self::parse_with(source, language).map(|_| ())
    }
}
