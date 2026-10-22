//! Dejavu core types: values, errors, i18n — no Oak.
//!
//! Parsing lives in `dejavu-language`; IR lives in `dejavu-ir`.

#![warn(missing_docs)]

pub mod errors;
pub mod i18n;
pub mod types;
pub mod values;

pub use errors::{CompileError, DejavuError, DejavuResult, RuntimeError};
pub use i18n::{SimpleTranslator, TranslationData, TranslationProvider};
pub use types::*;
pub use values::{Context, IntoValue, Value};
