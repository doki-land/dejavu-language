//! Axum adapter for Dejavu.
//!
//! Naming follows the **host** ecosystem (`axum-*`).
//! Core API remains the public [`dejavu`] crate (`Dejavu` facade).
//!
//! Errors are serialized as the shared Dejavu [`Diagnostic`] contract
//! (`application/json`), never as a flattened human string.

use axum::response::{Html, IntoResponse, Response};
use dejavu::{codes, parse_error_diagnostic, Diagnostic, Dejavu, IrDocument, IrError, Severity, Stage};
use serde_json::{Map, Value};

/// Error returned by adapter helpers.
#[derive(Debug, thiserror::Error)]
pub enum DejavuResponseError {
    #[error(transparent)]
    Ir(#[from] IrError),
}

impl DejavuResponseError {
    /// Structured diagnostic exported to the HTTP client.
    pub fn diagnostic(&self) -> Diagnostic {
        match self {
            DejavuResponseError::Ir(IrError::Diagnostic(d)) => d.clone(),
            DejavuResponseError::Ir(IrError::Json(e)) => {
                Diagnostic::new(Severity::Error, codes::IR_INVALID_JSON, Stage::Host).message(e.to_string())
            }
        }
    }
}

impl IntoResponse for DejavuResponseError {
    fn into_response(self) -> Response {
        let body = serde_json::to_string(&self.diagnostic())
            .unwrap_or_else(|e| format!("{{\"code\":\"{}\",\"message\":\"{}\"}}", codes::IR_INVALID_JSON, e));
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            [("content-type", "application/json")],
            body,
        )
            .into_response()
    }
}

/// Render an IR document with a JSON object context into `text/html`.
pub fn html_from_ir(doc: &IrDocument, ctx: &Value) -> Result<Html<String>, DejavuResponseError> {
    Ok(Html(Dejavu::render(doc, ctx)?))
}

/// Parse source → IR → HTML. Prefer `html_from_ir` in conformance-sensitive paths.
pub fn html_from_source(source: &str, ctx: &Value) -> Result<Html<String>, DejavuResponseError> {
    let doc = Dejavu::parse(source).map_err(|e| DejavuResponseError::from(IrError::from(parse_error_diagnostic(e))))?;
    html_from_ir(&doc, ctx)
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
