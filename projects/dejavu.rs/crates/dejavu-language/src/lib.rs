//! Dejavu T1 language frontend: hand-written lexer + recursive descent parser.
//!
//! Pipeline: `source → NativeTemplate → IrDocument` (via [`dejavu_ir::encode_native`]).
//!
//! This crate is **miette-free**. Error surface is plain `ParseError` + `ParseResult<T>`.
//! miette / fancy formatting — if needed — happens at host layers.

#![forbid(unsafe_code)]

mod error;
mod lexer;
mod parser;
mod token;

pub use error::{ParseError, ParseResult};
pub use parser::{parse, parse_named, parse_native, parse_with, parse_native_with};

/// Re-export IR document + Language types for convenience.
pub use dejavu_ir::{IrDocument, Language, NativeTemplate, default_language};
