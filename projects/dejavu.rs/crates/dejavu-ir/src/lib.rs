//! Dejavu IR v0: types, native AST encode/decode, normalize, and render.
//!
//! Parsing lives in [`dejavu_language`] (hand-written lexer + recursive descent + miette).

pub mod builtins;
pub mod catalog;
pub mod native;
pub mod normalize;
pub mod registry;
pub mod render;
pub mod trim;
pub mod types;

pub use builtins::{call_function, call_test};
pub use catalog::{
    CatalogFiles, CatalogLoader, CatalogRoot, ResolveHit, ResolveRoot, TemplateCatalog, canonical_id, resolve_ref,
};
pub use native::{NativeExpr, NativeImportName, NativeKind, NativeNode, NativeParam, NativeTemplate};
pub use normalize::normalize_value;
pub use trim::{level_of, normalized_children, trim_prefix, trim_suffix};
pub use registry::{Capability, CapabilitySet, CustomEntry, Namespace, validate_custom_entries};
pub use render::{
    CustomError, CustomRegistry, RenderOptions, ResolvedTemplate, TemplateLoader, render_ir, render_ir_opts,
    render_ir_with_loader,
};
pub use types::{IrDocument, IrNode, Language, TemplateConfig, IR_VERSION, default_language};

pub use dejavu_diagnostic::{ArgValue, Diagnostic, RelatedSpan, Severity, Span, Stage, codes};

use serde_json::Value;

/// Errors produced by IR encode/decode/render.
///
/// - `Json`：serde_json 错误（宿主输入 / IR wire 问题）
/// - `Diagnostic`：模板域错误的唯一形态（code + args + span + stage）
#[derive(Debug)]
pub enum IrError {
    Json(serde_json::Error),
    /// 结构化诊断（code + args + span + stage）。
    Diagnostic(Diagnostic),
}

impl std::fmt::Display for IrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IrError::Json(e) => write!(f, "json error: {e}"),
            IrError::Diagnostic(d) => d.fmt(f),
        }
    }
}

impl std::error::Error for IrError {}

impl From<serde_json::Error> for IrError {
    fn from(e: serde_json::Error) -> Self {
        IrError::Json(e)
    }
}

impl From<Diagnostic> for IrError {
    fn from(d: Diagnostic) -> Self {
        IrError::Diagnostic(d)
    }
}

impl IrError {
    /// 在 Diagnostic 变体上加一个 arg。
    /// 其他变体保持不变。
    pub fn arg(self, key: impl Into<String>, value: impl Into<dejavu_diagnostic::ArgValue>) -> Self {
        match self {
            IrError::Diagnostic(d) => IrError::Diagnostic(d.arg(key, value)),
            other => other,
        }
    }

    /// 在 Diagnostic 变体上加 span。
    pub fn span(self, start: usize, end: usize) -> Self {
        match self {
            IrError::Diagnostic(d) => IrError::Diagnostic(d.span(start, end)),
            other => other,
        }
    }

    /// 在 Diagnostic 变体上加 path。
    pub fn path(self, p: impl Into<String>) -> Self {
        match self {
            IrError::Diagnostic(d) => IrError::Diagnostic(d.path(p)),
            other => other,
        }
    }

    /// 在 Diagnostic 变体上加 template_id。
    pub fn template_id(self, id: impl Into<String>) -> Self {
        match self {
            IrError::Diagnostic(d) => IrError::Diagnostic(d.template_id(id)),
            other => other,
        }
    }
}

// ---------------------------------------------------------------------------
// Diagnostic → IrError 构造便捷函数
// ---------------------------------------------------------------------------

/// 快速构造 render 阶段的错误（直接返回 IrError，避免 blanket Into 歧义）。
pub fn diag_render(code: &str) -> IrError {
    IrError::Diagnostic(Diagnostic::new(Severity::Error, code, Stage::Render))
}

/// 快速构造 load 阶段的错误（直接返回 IrError，避免 blanket Into 歧义）。
pub fn diag_load(code: &str) -> IrError {
    IrError::Diagnostic(Diagnostic::new(Severity::Error, code, Stage::Load))
}

// ---------------------------------------------------------------------------
// 保持向下兼容的公开 API
// ---------------------------------------------------------------------------

/// Encode a native T1 AST into an IR document.
pub fn encode_native(template: &NativeTemplate) -> IrDocument {
    IrDocument { ir_version: IR_VERSION.into(), language: default_language(), body: native::encode_template(template) }
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
