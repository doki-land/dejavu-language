//! miette diagnostics for the T1 parser.

use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

/// A parse failure with source highlights.
#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
#[diagnostic(code(dejavu::parse))]
pub struct ParseError {
    #[source_code]
    pub src: NamedSource<String>,
    #[label("{label}")]
    pub span: SourceSpan,
    pub message: String,
    pub label: String,
}

impl ParseError {
    pub fn new(
        source: impl Into<String>,
        name: impl Into<String>,
        span: impl Into<SourceSpan>,
        message: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            src: NamedSource::new(name.into(), source.into()),
            span: span.into(),
            message: message.into(),
            label: label.into(),
        }
    }

    pub fn at(
        source: &str,
        name: &str,
        start: usize,
        len: usize,
        message: impl Into<String>,
        label: impl Into<String>,
    ) -> Self {
        let len = if len == 0 { 1.min(source.len().saturating_sub(start).max(1)) } else { len };
        Self::new(source.to_string(), name, (start, len), message, label)
    }
}

/// Result alias for language frontend APIs.
pub type ParseResult<T> = Result<T, miette::Error>;
