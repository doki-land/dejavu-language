//! Dejavu core types: values, errors, i18n — no Oak.
//!
//! Parsing: [`dejavu_language`]. IR: [`dejavu_ir`].

#![warn(missing_docs)]

pub mod errors;
pub mod i18n;
pub mod types;
pub mod values;

/// Parse helpers (hand-written lexer + RD via `dejavu-language`).
pub mod parser {
    use crate::{CompileError, DejavuError};
    use dejavu_ir::IrDocument;
    use dejavu_language::ParseError;

    /// Parse Dejavu T1 source into an IR document.
    ///
    /// Maps `dejavu_language::ParseError` → `DejavuError::CompileError`.
    /// The span is converted to (line, column) by counting `\n` from the
    /// raw source stored inside `ParseError`.
    pub fn parse(content: &str) -> Result<IrDocument, DejavuError> {
        dejavu_language::parse(content).map_err(parse_error_to_dejavu)
    }

    /// Alias kept for older call sites.
    pub fn parse_to_ir(content: &str) -> Result<IrDocument, DejavuError> {
        parse(content)
    }

    fn parse_error_to_dejavu(e: ParseError) -> DejavuError {
        let (line, column) = byte_to_line_col(&e.source, e.span.start);
        DejavuError::CompileError(CompileError::CodeGenError {
            message: e.message,
            line: Some(line),
            column: Some(column),
        })
    }

    /// Convert a UTF-8 byte offset into (1-based line, 1-based column).
    fn byte_to_line_col(source: &str, offset: usize) -> (usize, usize) {
        let mut line = 1usize;
        let mut col = 1usize;
        for (i, ch) in source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

pub use errors::{CompileError, DejavuError, DejavuResult, RuntimeError};
pub use i18n::{SimpleTranslator, TranslationData, TranslationProvider};
pub use types::*;
pub use values::{Context, IntoValue, Value};
