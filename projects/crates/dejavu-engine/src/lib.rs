//! Dejavu engine (Oak-free): parse via `dejavu-language`, render via `dejavu-ir`.
//!
//! No `miette` dependency — error surfaces are plain `IrError` / `ParseError`.
//! Host adapters (CLI fancy diagnostics, Axum JSON, Doki aggregator) build
//! their own presentation layer from those structs.

#![warn(missing_docs)]

use dejavu_ir::{IrDocument, IrError};
use dejavu_language::ParseResult;
use serde_json::Value;

pub use dejavu_ir;
pub use dejavu_ir::{render_ir_with_loader};
pub use dejavu_language;
pub use dejavu_types::{parser};

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Parse T1 source → IR.
pub fn parse(source: &str) -> ParseResult<IrDocument> {
    dejavu_language::parse(source)
}

/// Render an IR document with a JSON object context.
pub fn render(doc: &IrDocument, ctx: &Value) -> Result<String, IrError> {
    dejavu_ir::render_ir(doc, ctx)
}

/// Parse then render.
///
/// Parse failures are mapped into `IrError::Diagnostic` so callers get a
/// single error surface — no need for a combined enum.
pub fn render_source(source: &str, ctx: &Value) -> Result<String, IrError> {
    let doc = parse(source).map_err(parse_error_to_ir)?;
    dejavu_ir::render_ir(&doc, ctx)
}

fn parse_error_to_ir(e: dejavu_language::ParseError) -> IrError {
    let mut diag = dejavu_ir::Diagnostic::new(
        dejavu_ir::Severity::Error,
        e.code.unwrap_or(dejavu_ir::codes::PARSE_UNEXPECTED_TOKEN),
        dejavu_ir::Stage::Parse,
    )
    .path(e.file);
    if e.span.len() > 0 {
        diag = diag.span(e.span.start, e.span.end);
    }
    IrError::Diagnostic(diag)
}

/// Engine facade matching other language hosts.
pub struct DejavuEngine;

impl DejavuEngine {
    /// Parse source to IR.
    pub fn parse(&self, source: &str) -> ParseResult<IrDocument> {
        parse(source)
    }

    /// Render IR + context.
    pub fn render(&self, doc: &IrDocument, ctx: &Value) -> Result<String, IrError> {
        render(doc, ctx)
    }

    /// One-shot source render.
    pub fn render_source(&self, source: &str, ctx: &Value) -> Result<String, IrError> {
        render_source(source, ctx)
    }
}

impl Default for DejavuEngine {
    fn default() -> Self {
        Self
    }
}
