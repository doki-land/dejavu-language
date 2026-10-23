//! Dejavu AOT codegen: IR → ESM JavaScript.
//!
//! This crate is the upstream codegen stage that produces a JS module
//! consumable by `@doki-land/dejavu-engine/runtime`.
//!
//! # Scope (phase 1)
//!
//! * One template IR → one ESM module
//! * Text / Interpolation / If / ElseIf / For / Expr(Literal, Identifier,
//!   Member, Index, Binary, Unary, Pipe)
//! * `ExprCall` → compile-stage diagnostic + syntactic fallback
//! * `Stmt.Block` / `Stmt.Extends` / `Stmt.Include` / `Stmt.Super` → compile
//!   diagnostic + `throw new Error(...)` placeholder
//! * No source map emission (`null` for now)
//! * No catalog traversal (`precompileCatalog` follows later)

pub mod codegen;
pub mod emit;
pub mod options;

pub use codegen::{PrecompileResult, precompile_single};
pub use options::CodegenOptions;
