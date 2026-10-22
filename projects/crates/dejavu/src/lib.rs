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
    ArgValue, Diagnostic, IrDocument, IrError, IrNode, RelatedSpan, Severity, Span, Stage, TemplateConfig, codes,
    diag_load, diag_render, normalize_value,
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
    /// Parse source → IR.
    pub fn parse(source: &str) -> ParseResult<IrDocument> {
        parse(source)
    }

    /// Render IR + JSON context.
    pub fn render(doc: &IrDocument, ctx: &serde_json::Value) -> Result<String, IrError> {
        render(doc, ctx)
    }

    /// Parse then render.
    pub fn render_source(source: &str, ctx: &serde_json::Value) -> Result<String, IrError> {
        render_source(source, ctx)
    }

    /// Syntax check (parse only).
    pub fn check(source: &str) -> ParseResult<()> {
        parse(source).map(|_| ())
    }
}
