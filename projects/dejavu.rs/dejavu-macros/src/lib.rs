//! Procedural macros for DejaVu (Oak-free stub).
//!
//! Full AOT codegen via Oak was removed. Prefer runtime
//! `dejavu_language::parse` + `dejavu_ir::render_ir` until IR-based macros land.

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Placeholder `#[derive(Template)]` — emits a type with a stub `render` note.
#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    quote! {
        impl #name {
            /// Prefer the public crate: `dejavu::Dejavu::render_source`.
            pub fn render_hint() -> &'static str {
                "use dejavu::Dejavu::render_source"
            }
        }
    }
    .into()
}

/// Placeholder `template!` macro.
#[proc_macro]
pub fn template(input: TokenStream) -> TokenStream {
    let _ = input;
    quote! {
        compile_error!("template! macro requires IR-based rewrite; use dejavu::Dejavu::parse for now")
    }
    .into()
}
