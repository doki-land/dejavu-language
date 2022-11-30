//! Dejavu IR v1: types, native AST encode/decode, normalize, and render.
//!
//! Parsing lives in [`dejavu_language`] (hand-written lexer + recursive descent + miette).

pub mod native;
pub mod normalize;
pub mod render;
pub mod types;

pub use native::{NativeExpr, NativeNode, NativeTemplate};
pub use normalize::normalize_value;
pub use render::render_ir;
pub use types::{IrDocument, IrNode, Language, TemplateConfig, default_language};

use serde_json::Value;
use thiserror::Error;

/// Errors produced by IR encode/decode/render.
#[derive(Debug, Error)]
pub enum IrError {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("render error: {0}")]
    Render(String),
}

/// Encode a native T1 AST into an IR document.
pub fn encode_native(template: &NativeTemplate) -> IrDocument {
    IrDocument { ir_version: "1.0".into(), language: default_language(), body: native::encode_template(template) }
}

/// Decode IR JSON into typed IR, then to native AST.
pub fn decode_ir_json(json: &str) -> Result<NativeTemplate, IrError> {
    let doc: IrDocument = serde_json::from_str(json)?;
    Ok(native::decode_template(&doc.body))
}

/// Render IR JSON with a JSON object context.
pub fn render_ir_json(ir_json: &str, ctx: &Value) -> Result<String, IrError> {
    let doc: IrDocument = serde_json::from_str(ir_json)?;
    render_ir(&doc, ctx)
}

/// Normalize an IR JSON value for semantic equality.
pub fn normalize_ir_json(json: &str) -> Result<Value, IrError> {
    let value: Value = serde_json::from_str(json)?;
    Ok(normalize_value(value))
}
