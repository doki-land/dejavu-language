//! Minimal runtime utilities for DejaVu AOT / host helpers (`#![no_std]`).
#![warn(missing_docs)]
#![no_std]
extern crate alloc;

mod display;
pub mod escaper;
pub mod looper;
mod traits;

pub use crate::{
    display::{
        EscapeDisplay, Escaper, HtmlEscapeConfig, escape, escape_with_config, safe_html_attribute_escape, safe_html_escape,
    },
    traits::Template,
};
