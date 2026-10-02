//! Axum adapter for Dejavu.
//!
//! Naming follows the **host** ecosystem (`axum-*`).
//! Core API remains the public [`dejavu`] crate (`Dejavu` facade).

use axum::response::{Html, IntoResponse, Response};
use dejavu::{Dejavu, IrDocument, IrError};
use serde_json::{Map, Value};

/// Error returned by adapter helpers.
#[derive(Debug, thiserror::Error)]
pub enum DejavuResponseError {
    #[error(transparent)]
    Ir(#[from] IrError),
    #[error("{0}")]
    Parse(String),
}

impl IntoResponse for DejavuResponseError {
    fn into_response(self) -> Response {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

/// Render an IR document with a JSON object context into `text/html`.
pub fn html_from_ir(doc: &IrDocument, ctx: &Value) -> Result<Html<String>, DejavuResponseError> {
    Ok(Html(Dejavu::render(doc, ctx)?))
}

/// Parse source → IR → HTML. Prefer `html_from_ir` in conformance-sensitive paths.
pub fn html_from_source(source: &str, ctx: &Value) -> Result<Html<String>, DejavuResponseError> {
    Dejavu::render_source(source, ctx).map(Html).map_err(|e| DejavuResponseError::Parse(format!("{e:?}")))
}

/// Build a JSON object context from an iterator of `(key, value)`.
pub fn context_from_iter<I, K>(iter: I) -> Value
where
    I: IntoIterator<Item = (K, Value)>,
    K: Into<String>,
{
    let mut map = Map::new();
    for (k, v) in iter {
        map.insert(k.into(), v);
    }
    Value::Object(map)
}

/// Axum handler-friendly: render IR JSON string + context map → HTML.
pub fn html_from_ir_json(ir_json: &str, ctx: &Value) -> Result<Html<String>, DejavuResponseError> {
    let doc: IrDocument = serde_json::from_str(ir_json).map_err(IrError::from)?;
    html_from_ir(&doc, ctx)
}
