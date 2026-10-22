//! Dejavu N-API 引擎面。
//!
//! 这里只导出**正交、可组合的引擎能力**：解析源码、规范化 IR、渲染 IR / 源码、
//! 解析模板引用，以及结构化诊断。命令语义（`build` / `dev` / `check` 的参数、
//! 配置、stdio、退出码、交互）属于 TypeScript `src/cli/`，不在本 crate 内。
//!
//! 绑定合同：返回 [`BINDING_CONTRACT`]，宿主必须先核对版本再使用能力。
//! 该版本只随导出面破坏性变化而递增，与 npm 包版本无关。
//!
//! 所有 JSON 型输入（context / catalog / options）都用字符串传递，
//! 避免 N-API 对象与 `serde_json::Value` 之间的隐式类型分歧。

#![warn(missing_docs)]

use std::collections::BTreeMap;

use dejavu_diagnostic::{Diagnostic, Severity, Stage, codes};
use dejavu_ir::{
    CatalogRoot, IrDocument, IrError, RenderOptions, ResolveRoot, TemplateCatalog, TemplateLoader, render_ir_opts, resolve_ref,
};
use napi_derive::napi;
use serde::Deserialize;
use serde_json::Value;

/// 绑定合同标识。
pub const BINDING_CONTRACT: &str = "dejavu.napi/1";

/// path-v1 默认扩展名表（与 catalog loader 一致）。
const DEFAULT_EXTENSIONS: [&str; 4] = ["", ".html", ".doki", ".dejavu"];

// ---------------------------------------------------------------------------
// 结构化诊断 wire
// ---------------------------------------------------------------------------

/// 源码区间（UTF-8 byte offset，`end` exclusive）。
#[napi(object)]
pub struct JsSpan {
    /// 起始 byte offset。
    pub start: u32,
    /// 结束 byte offset（exclusive）。
    pub end: u32,
}

/// 相关位置。
#[napi(object)]
pub struct JsRelatedSpan {
    /// 起始 byte offset。
    pub start: u32,
    /// 结束 byte offset。
    pub end: u32,
    /// 该位置的含义。
    pub label: String,
    /// 可选 source path。
    pub path: Option<String>,
}

/// 跨宿主诊断。
#[napi(object)]
pub struct JsDiagnostic {
    /// 稳定机器身份，例如 `dejavu.template.template-not-found`。
    pub code: String,
    /// `error` / `warning` / `info` / `debug`。
    pub severity: String,
    /// `parse` / `load` / `resolve` / `render` / `compile` / `config` / `host`。
    pub stage: String,
    /// 无 catalog 时的 fallback 文案，可能为空。
    pub message: String,
    /// canonical source path。
    pub path: Option<String>,
    /// canonical template id。
    pub template_id: Option<String>,
    /// 主 span。
    pub span: Option<JsSpan>,
    /// 结构化 args 的 JSON 对象字符串（翻译优先用 code + args）。
    pub args_json: String,
    /// 相关 span。
    pub related: Vec<JsRelatedSpan>,
}

// ---------------------------------------------------------------------------
// 请求 / 响应
// ---------------------------------------------------------------------------

/// `parseIr` 请求。
#[napi(object)]
pub struct ParseRequest {
    /// 模板源码。
    pub source: String,
    /// 用于诊断的 canonical path。
    pub file: Option<String>,
    /// 保留字段：当前引擎只有 T1 语法。
    pub language: Option<String>,
}

/// IR 响应。
#[napi(object)]
pub struct IrResponse {
    /// 成功时的 IR JSON。
    pub ir: Option<String>,
    /// 全部诊断（error 级表示失败）。
    pub diagnostics: Vec<JsDiagnostic>,
}

/// `normalizeIr` 请求。
#[napi(object)]
pub struct NormalizeRequest {
    /// IR JSON。
    pub ir: String,
}

/// `renderIr` / `renderSource` 请求。
#[napi(object)]
pub struct RenderRequest {
    /// IR JSON（`renderSource` 时忽略）。
    pub ir: String,
    /// 渲染 context 的 JSON 对象字符串。
    pub context: String,
    /// `{ strict?, name? }` JSON。
    pub options: Option<String>,
    /// catalog 规格 JSON。
    pub catalog: Option<String>,
}

/// `renderSource` 请求。
#[napi(object)]
pub struct RenderSourceRequest {
    /// 模板源码。
    pub source: String,
    /// 渲染 context 的 JSON 对象字符串。
    pub context: String,
    /// `{ strict?, name? }` JSON。
    pub options: Option<String>,
    /// catalog 规格 JSON。
    pub catalog: Option<String>,
}

/// 渲染响应。
#[napi(object)]
pub struct RenderResponse {
    /// 成功时的渲染结果。
    pub output: Option<String>,
    /// 全部诊断。
    pub diagnostics: Vec<JsDiagnostic>,
}

/// `resolveTemplateRef` 的 root 输入。
#[napi(object)]
pub struct JsResolveRoot {
    /// root 名称。
    pub name: String,
    /// scheme，缺省等于 name。
    pub scheme: Option<String>,
    /// 优先级，越大越先命中。
    pub priority: Option<i32>,
    /// root 内全部 canonical 路径。
    pub files: Vec<String>,
}

/// `resolveTemplateRef` 请求。
#[napi(object)]
pub struct ResolveRequest {
    /// 模板引用。
    pub reference: String,
    /// 引用来源的 canonical id。
    pub from: Option<String>,
    /// 扩展名探测表。
    pub extensions: Option<Vec<String>>,
    /// roots 索引。
    pub roots: Option<Vec<JsResolveRoot>>,
}

/// 引用解析响应。
#[napi(object)]
pub struct ResolveResponse {
    /// 命中时的 canonical id（`root:path`）。
    pub id: Option<String>,
    /// 全部诊断。
    pub diagnostics: Vec<JsDiagnostic>,
}

// ---------------------------------------------------------------------------
// 能力导出
// ---------------------------------------------------------------------------

/// 返回绑定合同标识；宿主应校验后使用其余能力。
#[napi(js_name = "bindingContractVersion")]
pub fn binding_contract_version() -> String {
    BINDING_CONTRACT.to_string()
}

/// 解析模板源码为 IR。
#[napi(js_name = "parseIr")]
pub fn parse_ir(request: ParseRequest) -> IrResponse {
    let parsed = match request.file.as_deref() {
        Some(file) => dejavu_language::parse_named(&request.source, file),
        None => dejavu_language::parse(&request.source),
    };

    match parsed {
        Ok(document) => match serde_json::to_string(&document) {
            Ok(ir) => IrResponse { ir: Some(ir), diagnostics: Vec::new() },
            Err(error) => IrResponse {
                ir: None,
                diagnostics: vec![to_js(
                    &Diagnostic::new(Severity::Error, codes::IR_INVALID_JSON, Stage::Host).message(error.to_string()),
                )],
            },
        },
        Err(error) => IrResponse { ir: None, diagnostics: vec![to_js(&dejavu_engine::parse_error_diagnostic(error))] },
    }
}

/// 规范化 IR JSON（丢弃 span / 空文本等不影响语义的形状）。
#[napi(js_name = "normalizeIr")]
pub fn normalize_ir(request: NormalizeRequest) -> IrResponse {
    match dejavu_ir::normalize_ir_json(&request.ir) {
        Ok(value) => match serde_json::to_string(&value) {
            Ok(ir) => IrResponse { ir: Some(ir), diagnostics: Vec::new() },
            Err(error) => failure_ir(error.to_string()),
        },
        Err(error) => failure_ir_error(error),
    }
}

/// 渲染 IR JSON。
#[napi(js_name = "renderIr")]
pub fn render_ir_api(request: RenderRequest) -> RenderResponse {
    let document = match serde_json::from_str::<IrDocument>(&request.ir) {
        Ok(document) => document,
        Err(error) => return RenderResponse { output: None, diagnostics: vec![to_js(&json_diag(error.to_string()))] },
    };
    render_document(&document, &request.context, request.options.as_deref(), request.catalog.as_deref())
}

/// 解析并渲染模板源码。
#[napi(js_name = "renderSource")]
pub fn render_source_api(request: RenderSourceRequest) -> RenderResponse {
    let document = match request.name_file() {
        Some(file) => dejavu_language::parse_named(&request.source, &file),
        None => dejavu_language::parse(&request.source),
    };
    let document = match document {
        Ok(document) => document,
        Err(error) => {
            return RenderResponse { output: None, diagnostics: vec![to_js(&dejavu_engine::parse_error_diagnostic(error))] };
        }
    };
    render_document(&document, &request.context, request.options.as_deref(), request.catalog.as_deref())
}

impl RenderSourceRequest {
    fn name_file(&self) -> Option<String> {
        self.options.as_deref().and_then(|raw| serde_json::from_str::<RenderOptionsWire>(raw).ok()).and_then(|wire| wire.name)
    }
}

/// 纯 path-v1 引用解析：不读文件、不解析模板。
#[napi(js_name = "resolveTemplateRef")]
pub fn resolve_template_ref(request: ResolveRequest) -> ResolveResponse {
    let roots: Vec<ResolveRoot> = request
        .roots
        .unwrap_or_default()
        .into_iter()
        .map(|root| ResolveRoot {
            scheme: root.scheme.unwrap_or_else(|| root.name.clone()),
            name: root.name,
            priority: root.priority.unwrap_or(0),
            files: root.files,
        })
        .collect();
    let extensions = request.extensions.unwrap_or_else(|| DEFAULT_EXTENSIONS.iter().map(|s| s.to_string()).collect());

    match resolve_ref(&roots, &extensions, &request.reference, request.from.as_deref()) {
        Ok(hit) => ResolveResponse {
            id: roots.get(hit.root_index).map(|root| dejavu_ir::canonical_id(&root.name, &hit.path)),
            diagnostics: Vec::new(),
        },
        Err(diagnostic) => ResolveResponse { id: None, diagnostics: vec![to_js(&diagnostic)] },
    }
}

// ---------------------------------------------------------------------------
// 内部实现
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenderOptionsWire {
    #[serde(default)]
    strict: bool,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogSpec {
    #[serde(default)]
    roots: Vec<CatalogRootSpec>,
    #[serde(default)]
    extensions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogRootSpec {
    name: String,
    #[serde(default)]
    scheme: Option<String>,
    #[serde(default)]
    priority: i32,
    #[serde(default)]
    files: BTreeMap<String, String>,
    #[serde(default)]
    base_path: Option<String>,
}

fn render_document(document: &IrDocument, context: &str, options: Option<&str>, catalog: Option<&str>) -> RenderResponse {
    let context: Value = match serde_json::from_str(context) {
        Ok(value) => value,
        Err(error) => return RenderResponse { output: None, diagnostics: vec![to_js(&json_diag(error.to_string()))] },
    };

    let wire = match options {
        Some(raw) => match serde_json::from_str::<RenderOptionsWire>(raw) {
            Ok(wire) => wire,
            Err(error) => {
                return RenderResponse { output: None, diagnostics: vec![to_js(&json_diag(error.to_string()))] };
            }
        },
        None => RenderOptionsWire::default(),
    };

    let mut catalog_state = None;
    if let Some(raw) = catalog {
        match build_catalog(raw) {
            Ok(value) => catalog_state = Some(value),
            Err(diagnostic) => return RenderResponse { output: None, diagnostics: vec![to_js(&diagnostic)] },
        }
    }

    let options = RenderOptions { strict_undefined: wire.strict, name: wire.name };
    let mut loader_state = catalog_state.as_ref().map(dejavu_ir::CatalogLoader::new);
    let loader: Option<&mut dyn TemplateLoader> = match loader_state.as_mut() {
        Some(loader) => Some(loader),
        None => None,
    };
    let result = render_ir_opts(document, &context, options, loader);

    match result {
        Ok(output) => RenderResponse { output: Some(output), diagnostics: Vec::new() },
        Err(error) => RenderResponse { output: None, diagnostics: vec![to_js(&ir_error_diagnostic(error))] },
    }
}

fn build_catalog(raw: &str) -> Result<TemplateCatalog, Diagnostic> {
    let spec: CatalogSpec = serde_json::from_str(raw).map_err(|error| json_diag(format!("invalid catalog json: {error}")))?;

    let mut roots = Vec::with_capacity(spec.roots.len());
    for spec_root in spec.roots {
        let mut root = CatalogRoot::new(
            spec_root.name.clone(),
            spec_root.scheme.unwrap_or_else(|| spec_root.name.clone()),
            spec_root.priority,
        );
        root.base_path = spec_root.base_path;
        for (path, ir) in spec_root.files {
            let document: IrDocument = serde_json::from_str(&ir).map_err(|error| {
                Diagnostic::new(Severity::Error, codes::IR_INVALID_JSON, Stage::Load)
                    .message(format!("invalid IR for template '{path}': {error}"))
                    .path(path.clone())
            })?;
            root.insert(path, document);
        }
        roots.push(root);
    }

    let extensions = spec.extensions.unwrap_or_else(|| DEFAULT_EXTENSIONS.iter().map(|s| s.to_string()).collect());
    Ok(TemplateCatalog::new(roots, extensions))
}

fn failure_ir(message: String) -> IrResponse {
    IrResponse { ir: None, diagnostics: vec![to_js(&json_diag(message))] }
}

fn failure_ir_error(error: IrError) -> IrResponse {
    IrResponse { ir: None, diagnostics: vec![to_js(&ir_error_diagnostic(error))] }
}

fn json_diag(message: String) -> Diagnostic {
    Diagnostic::new(Severity::Error, codes::IR_INVALID_JSON, Stage::Host).message(message)
}

fn ir_error_diagnostic(error: IrError) -> Diagnostic {
    match error {
        IrError::Diagnostic(diagnostic) => diagnostic,
        IrError::Json(error) => json_diag(error.to_string()),
    }
}

fn to_js(diagnostic: &Diagnostic) -> JsDiagnostic {
    JsDiagnostic {
        code: diagnostic.code.clone(),
        severity: severity_name(diagnostic.severity).to_string(),
        stage: stage_name(diagnostic.stage).to_string(),
        message: diagnostic.message.clone().unwrap_or_default(),
        path: diagnostic.path.clone(),
        template_id: diagnostic.template_id.clone(),
        span: diagnostic.span.map(|span| JsSpan { start: span.start as u32, end: span.end as u32 }),
        args_json: serde_json::to_string(&diagnostic.args).unwrap_or_else(|_| "{}".to_string()),
        related: diagnostic
            .related
            .iter()
            .map(|related| JsRelatedSpan {
                start: related.span.start as u32,
                end: related.span.end as u32,
                label: related.label.clone(),
                path: related.path.clone(),
            })
            .collect(),
    }
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Debug => "debug",
    }
}

fn stage_name(stage: Stage) -> &'static str {
    match stage {
        Stage::Parse => "parse",
        Stage::Load => "load",
        Stage::Resolve => "resolve",
        Stage::Render => "render",
        Stage::Compile => "compile",
        Stage::Config => "config",
        Stage::Host => "host",
    }
}
