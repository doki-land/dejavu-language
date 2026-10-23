//! Dejavu AOT codegen: IR → ESM JavaScript.
//!
//! This crate is the upstream codegen stage that produces a JS module
//! consumable by `@doki-land/dejavu-engine/runtime`.
//!
//! # Scope
//!
//! * One template IR → one ESM module (`precompile_single`), or a whole
//!   catalog of templates → multiple ESM modules plus a manifest
//!   (`precompile_catalog`).
//! * Text / Interpolation / If / ElseIf / For / Expr(Literal, Identifier,
//!   Member, Index, Binary, Unary, Pipe)
//! * `ExprCall` → compile-stage diagnostic + syntactic fallback
//! * `Stmt.Block` / `Stmt.Extends` / `Stmt.Include` / `Stmt.Super` → handled
//!   at build time in catalog mode; fall back to a compile diagnostic in
//!   single-template mode
//! * No source map emission (`null` for now)

pub mod catalog;
pub mod codegen;
pub mod emit;
pub mod options;

pub use codegen::{PrecompileCatalogResult, PrecompileResult, GeneratedModule, precompile_catalog, precompile_single};
pub use options::CodegenOptions;
