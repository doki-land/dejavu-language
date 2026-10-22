//! Parser errors — plain struct, no miette coupling.
//!
//! miette 适配（如果仍需要 source snippet 高亮）移到宿主层。
//! `dejavu_diagnostic::Span` 是跨 crate 的中立合同（`start..end` exclusive，UTF-8 byte offset）。

use dejavu_diagnostic::Span;
use std::fmt;

/// A parse failure with source range.
///
/// Kept small and data-only — no `miette::Diagnostic` derive, no `NamedSource`
/// wrapper. miette / @vmz/diagnostic adapters construct their own presentation
/// from the raw fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Raw source text (for source snippets if the adapter needs it).
    pub source: String,
    /// Canonical file name for this source (e.g. `"template.dejavu"`).
    pub file: String,
    /// Byte range of the offending token / region (`start..end` exclusive).
    pub span: Span,
    /// Short human-readable error (no source context).
    pub message: String,
    /// Label for the span when rendered (e.g. `"unexpected token"`).
    pub label: String,
    /// Stable machine-readable code, e.g. `dejavu.parse.unclosed-block`.
    pub code: Option<&'static str>,
}

impl ParseError {
    pub fn new(
        source: impl Into<String>,
        name: impl Into<String>,
        span: impl Into<Span>,
        message: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            source: source.into(),
            file: name.into(),
            span: span.into(),
            message: message.into(),
            label: label.into(),
            code: None,
        }
    }

    /// Construct a `ParseError` from a byte offset + length.
    ///
    /// Call sites use this exclusively; signature is intentionally stable so
    /// parser sub-modules (`expr.rs`, `stmt.rs`, `stmt_head.rs`) never need to
    /// know about `Span` or any diagnostic backend.
    pub fn at(
        source: &str,
        name: &str,
        start: usize,
        len: usize,
        message: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        let clamped = if len == 0 {
            1.min(source.len().saturating_sub(start).max(1))
        } else {
            len
        };
        Self::new(source.to_string(), name, Span::new(start, start + clamped), message, label)
    }

    /// Attach a stable error code (builder-style).
    pub fn with_code(mut self, code: &'static str) -> Self {
        self.code = Some(code);
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}..{}: {}",
            self.file, self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// Result alias for parser-facing APIs.
///
/// Previously `Result<T, miette::Error>` — the parser no longer depends on
/// miette. Hosts that want fancy diagnostics adapt `ParseError` themselves.
pub type ParseResult<T> = Result<T, ParseError>;
