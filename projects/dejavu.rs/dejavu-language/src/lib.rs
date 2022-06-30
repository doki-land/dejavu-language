//! Dejavu T1 language frontend: hand-written lexer + recursive descent + **miette**.
//!
//! Pipeline: `source → NativeTemplate → IrDocument` (via [`dejavu_ir::encode_native`]).

#![forbid(unsafe_code)]

mod error;
mod lexer;
mod parser;
mod token;

pub use error::{ParseError, ParseResult};
pub use parser::{parse, parse_named, parse_native};

/// Re-export IR document type for convenience.
pub use dejavu_ir::{IrDocument, NativeTemplate};
