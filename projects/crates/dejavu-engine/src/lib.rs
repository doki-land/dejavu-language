//! Dejavu engine (Oak-free): parse via `dejavu-language`, render via `dejavu-ir`.

#![warn(missing_docs)]

use dejavu_ir::{IrDocument, IrError, Severity, Diagnostic, render_ir};
use dejavu_language::ParseResult;
use serde_json::Value;

pub use dejavu_ir;
pub use dejavu_ir::{TemplateLoader, render_ir_with_loader};
pub use dejavu_language;
pub use dejavu_types::{errors::*, parser};

// ---------------------------------------------------------------------------
// Diagnostic → miette adapter
// ---------------------------------------------------------------------------

/// Convert a Dejavu `Diagnostic` into a `miette::Error`.
///
/// Builds a small wrapper implementing `miette::Diagnostic` so severity, code,
/// and structured args survive the boundary.
pub fn diagnostic_to_miette(d: Diagnostic) -> miette::Error {
    miette::Error::new(DejavuMietteDiag { diagnostic: d })
}

/// Convert any `IrError` to `miette::Error`, preserving structured fields.
///
/// # Why not `e.to_string()`?
///
/// The old `map_err(|e| miette::Error::msg(e.to_string()))` call silently
/// drops code, severity, span, and args — exactly the fields we just added.
/// This adapter keeps them so terminal users see the same identity and
/// argument values that machine consumers (Doki, Node CLI) see.
pub fn ir_error_to_miette(err: IrError) -> miette::Error {
    match err {
        IrError::Json(e) => miette::Error::msg(format!("json error: {e}")),
        IrError::Render(msg) => {
            // Legacy path kept only for compatibility; render.rs no longer uses it.
            miette::Error::msg(msg)
        }
        IrError::Diagnostic(d) => diagnostic_to_miette(d),
    }
}

/// Adapter wrapper implementing `miette::Diagnostic`.
#[derive(Debug)]
struct DejavuMietteDiag {
    diagnostic: Diagnostic,
}

impl std::fmt::Display for DejavuMietteDiag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.diagnostic.code)
    }
}

impl std::error::Error for DejavuMietteDiag {}

impl miette::Diagnostic for DejavuMietteDiag {
    fn severity(&self) -> Option<miette::Severity> {
        Some(match self.diagnostic.severity {
            Severity::Error => miette::Severity::Error,
            Severity::Warning => miette::Severity::Warning,
            Severity::Info | Severity::Debug => miette::Severity::Advice,
        })
    }

    fn code(&self) -> Option<Box<dyn std::fmt::Display>> {
        Some(Box::new(self.diagnostic.code.clone()))
    }

    fn help(&self) -> Option<Box<dyn std::fmt::Display>> {
        if self.diagnostic.args.is_empty() {
            None
        } else {
            let args_str = serde_json::to_string(&self.diagnostic.args).unwrap_or_default();
            Some(Box::new(format!("args: {args_str}")))
        }
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        // Intentionally not providing source snippet here — the engine crate
        // does not own the source string (it lives in the caller or Doki).
        // Path is surfaced through `url`.
        None
    }

    fn url(&self) -> Option<Box<dyn std::fmt::Display>> {
        self.diagnostic
            .template_id
            .as_ref()
            .or(self.diagnostic.path.as_ref())
            .map(|p| Box::new(p.clone()) as Box<dyn std::fmt::Display>)
    }
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

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
    render_ir(&doc, ctx).map_err(ir_error_to_miette)
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
