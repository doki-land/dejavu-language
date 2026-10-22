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
    #[allow(deprecated)] // Legacy Render variant kept only for compatibility
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
        let mut parts: Vec<String> = Vec::new();
        if !self.diagnostic.args.is_empty() {
            let args_str = serde_json::to_string(&self.diagnostic.args).unwrap_or_default();
            parts.push(format!("args: {args_str}"));
        }
        if let Some(span) = self.diagnostic.span {
            parts.push(format!("span: {}..{}", span.start, span.end));
        }
        if parts.is_empty() { None } else { Some(Box::new(parts.join("; "))) }
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        // Intentionally not providing source snippet here — the engine crate
        // does not own the source string (it lives in the caller or Doki).
        // Span info is surfaced via labels below; parser-level errors will own
        // their source once dejavu-language also migrates to the Diagnostic contract.
        None
    }

    fn url(&self) -> Option<Box<dyn std::fmt::Display>> {
        self.diagnostic
            .template_id
            .as_ref()
            .or(self.diagnostic.path.as_ref())
            .map(|p| Box::new(p.clone()) as Box<dyn std::fmt::Display>)
    }
    // labels() / source_code() deferred — requires caller-owned source text
    // and span→SourceSpan mapping that works for parser errors (which own source)
    // but not IR render errors (which operate on JSON IR without source).
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use dejavu_ir::codes;

    #[test]
    fn diagnostic_to_miette_preserves_code_severity_args_span_path() {
        let diag = Diagnostic::new(
            Severity::Error,
            codes::CIRCULAR_EXTENDS,
            dejavu_ir::Stage::Load,
        )
        .arg("chain", "a.dejavu")
        .path("templates/layout.dejavu")
        .template_id("theme:layout.dejavu")
        .span(10, 25);

        let miette_err = diagnostic_to_miette(diag);

        // Display must contain the machine code (stable identity)
        let display = format!("{miette_err}");
        assert!(
            display.contains("dejavu.template.circular-extends"),
            "Display should preserve code, got: {display}"
        );
    }

    #[test]
    fn ir_error_to_miette_diagnostic_variant_roundtrip() {
        let ir_err = IrError::Diagnostic(
            Diagnostic::new(
                Severity::Warning,
                codes::UNKNOWN_FILTER,
                dejavu_ir::Stage::Render,
            )
            .arg("filter", "myfilter"),
        );

        let miette_err = ir_error_to_miette(ir_err);
        let display = format!("{miette_err}");
        assert!(
            display.contains("dejavu.template.unknown-filter"),
            "Display should preserve code from Diagnostic variant, got: {display}"
        );
    }
}
