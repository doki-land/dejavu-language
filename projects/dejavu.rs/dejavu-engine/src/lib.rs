//! Dejavu engine (Oak-free): parse via `dejavu-language`, render via `dejavu-ir`.

#![warn(missing_docs)]

use dejavu_ir::{IrDocument, IrError, render_ir};
use dejavu_language::ParseResult;
use serde_json::Value;

pub use dejavu_ir;
pub use dejavu_language;
pub use dejavu_types::{errors::*, parser};

/// Parse T1 source → IR.
pub fn parse(source: &str) -> ParseResult<IrDocument> {
    dejavu_language::parse(source)
}

/// Render an IR document with a JSON object context.
pub fn render(doc: &IrDocument, ctx: &Value) -> Result<String, IrError> {
    render_ir(doc, ctx)
}

/// Parse then render.
pub fn render_source(source: &str, ctx: &Value) -> ParseResult<String> {
    let doc = parse(source)?;
    render_ir(&doc, ctx).map_err(|e| miette::Error::msg(e.to_string()))
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
    pub fn render_source(&self, source: &str, ctx: &Value) -> ParseResult<String> {
        render_source(source, ctx)
    }
}

impl Default for DejavuEngine {
    fn default() -> Self {
        Self
    }
}
