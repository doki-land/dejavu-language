//! Escaper module for escaping content in different formats
pub use self::{html::Html, text::Text};
use crate::Escaper;
use core::fmt::Write;

mod html;
mod text;
