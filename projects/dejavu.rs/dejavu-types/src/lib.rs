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

    /// Parse Dejavu T1 source into an IR document.
    pub fn parse(content: &str) -> Result<IrDocument, DejavuError> {
        dejavu_language::parse(content).map_err(|e| {
            DejavuError::CompileError(CompileError::CodeGenError { message: format!("{e:?}"), line: None, column: None })
        })
    }

    /// Alias kept for older call sites.
    pub fn parse_to_ir(content: &str) -> Result<IrDocument, DejavuError> {
        parse(content)
    }
}

pub use errors::{CompileError, DejavuError, DejavuResult, RuntimeError};
pub use i18n::{SimpleTranslator, TranslationData, TranslationProvider};
pub use types::*;
pub use values::{Context, IntoValue, Value};
