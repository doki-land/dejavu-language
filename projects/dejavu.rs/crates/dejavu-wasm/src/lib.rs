//! Dejavu WASM 语言桥。
//!
//! 面向 `wasm32-unknown-unknown`（浏览器 / Worker）导出与 N-API 等价的正交引擎能力：
//! 解析源码、规范化 IR、渲染 IR / 源码、解析模板引用，以及结构化诊断。
//!
//! 绑定合同：返回 [`BINDING_CONTRACT`]，宿主必须先核对版本再使用能力。
//!
//! 所有输入输出都是 JSON 字符串（请求 / 响应），宿主包装层只做 JSON 编解码。
//! 本 crate 不依赖文件系统、进程或时间，全部能力都是纯计算。

use std::collections::BTreeMap;

use dejavu_diagnostic::{Diagnostic, Severity, Stage, codes};
use dejavu_ir::{
    CatalogRoot, IrDocument, IrError, RenderOptions, ResolveRoot, TemplateCatalog, TemplateLoader, render_ir_opts, resolve_ref,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use wasm_bindgen::prelude::*;

/// 绑定合同标识。
pub const BINDING_CONTRACT: &str = "dejavu.wasm/1";

/// path-v1 默认扩展名表（与 catalog loader 一致）。
const DEFAULT_EXTENSIONS: [&str; 4] = ["", ".html", ".doki", ".dejavu"];

// ---------------------------------------------------------------------------
// 结构化诊断 wire
// ---------------------------------------------------------------------------

/// 源码区间（UTF-8 byte offset，`end` exclusive）。
#[derive(Debug, Serialize)]
struct SpanWire {
    start: u32,
    end: u32,
}

/// 相关位置。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelatedSpanWire {
    start: u32,
    end: u32,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

/// 跨宿主诊断 wire。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiagnosticWire {
    code: String,
    severity: String,
    stage: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    span: Option<SpanWire>,
    args_json: String,
    related: Vec<RelatedSpanWire>,
}

// ---------------------------------------------------------------------------
// 请求 / 响应 wire
// ---------------------------------------------------------------------------

/// `parseIr` 请求。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParseRequest {
    source: String,
    #[serde(default)]
    file: Option<String>,
    /// 保留字段：当前引擎只有 T1 语法。
    #[serde(default)]
    #[allow(dead_code)]
    language: Option<String>,
    /// 可选的 Language JSON，覆盖默认 delimiters（`<% %>` / `<# #>`）。
    #[serde(default)]
    delimiters: Option<String>,
}

/// IR 响应。
#[derive(Debug, Serialize)]
struct IrResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    ir: Option<String>,
    diagnostics: Vec<DiagnosticWire>,
}

/// `normalizeIr` 请求。
#[derive(Debug, Deserialize)]
struct NormalizeRequest {
    ir: String,
}

/// `renderIr` 请求。
#[derive(Debug, Deserialize)]
struct RenderRequest {
    ir: String,
    context: String,
    #[serde(default)]
    options: Option<String>,
    #[serde(default)]
    catalog: Option<String>,
}

/// `renderSource` 请求。
#[derive(Debug, Deserialize)]
struct RenderSourceRequest {
    source: String,
    context: String,
    #[serde(default)]
    options: Option<String>,
    #[serde(default)]
    catalog: Option<String>,
    #[serde(default)]
    delimiters: Option<String>,
}

/// 渲染响应。
#[derive(Debug, Serialize)]
struct RenderResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    output: Option<String>,
    diagnostics: Vec<DiagnosticWire>,
}

/// `resolveTemplateRef` 的 root 输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResolveRootInput {
    name: String,
    #[serde(default)]
    scheme: Option<String>,
    #[serde(default)]
    priority: Option<i32>,
    files: Vec<String>,
}

/// `resolveTemplateRef` 请求。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResolveRequest {
    reference: String,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    extensions: Option<Vec<String>>,
    #[serde(default)]
    roots: Option<Vec<ResolveRootInput>>,
}

/// 引用解析响应。
#[derive(Debug, Serialize)]
struct ResolveResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    diagnostics: Vec<DiagnosticWire>,
}

/// `precompileIrSingle` 请求。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrecompileSingleRequest {
    ir: String,
    #[serde(default)]
    options: Option<String>,
}

/// 一个生成行起点及其模板 span。
#[derive(Debug, Serialize)]
struct GeneratedOriginWire {
    line: u32,
    start: u32,
    end: u32,
}

/// AOT 生成的模块描述。
#[derive(Debug, Serialize)]
struct GeneratedModuleWire {
    path: String,
    code: String,
    dependencies: Vec<String>,
    origins: Vec<GeneratedOriginWire>,
}

/// 单模板 AOT 响应。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PrecompileSingleResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    module: Option<GeneratedModuleWire>,
    diagnostics: Vec<DiagnosticWire>,
    entry_id: String,
}

/// `precompileCatalog` 请求。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrecompileCatalogRequest {
    entry_ir: String,
    catalog: String,
    #[serde(default)]
    options: Option<String>,
}

/// Catalog AOT 响应。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PrecompileCatalogResponse {
    modules: Vec<GeneratedModuleWire>,
    #[serde(skip_serializing_if = "Option::is_none")]
    manifest: Option<String>,
    diagnostics: Vec<DiagnosticWire>,
    entry_id: String,
}

// ---------------------------------------------------------------------------
// 能力导出
// ---------------------------------------------------------------------------

/// 返回绑定合同标识；宿主应校验后使用其余能力。
#[wasm_bindgen(js_name = "bindingContractVersion")]
pub fn binding_contract_version() -> String {
    BINDING_CONTRACT.to_string()
}

/// 解析模板源码为 IR。
#[wasm_bindgen(js_name = "parseIr")]
pub fn parse_ir(request_json: &str) -> String {
    let request: ParseRequest = match from_request(request_json) {
        Ok(request) => request,
        Err(diagnostic) => return respond(&IrResponse { ir: None, diagnostics: vec![diagnostic] }),
    };
    let language = request
        .delimiters
        .as_deref()
        .and_then(|s| serde_json::from_str::<dejavu_ir::Language>(s).ok())
        .unwrap_or_else(dejavu_ir::default_language);
    let file = request.file.as_deref().unwrap_or("template.dejavu");
    let parsed = dejavu_language::parse_with(&request.source, file, &language);

    match parsed {
        Ok(document) => match serde_json::to_string(&document) {
            Ok(ir) => respond(&IrResponse { ir: Some(ir), diagnostics: Vec::new() }),
            Err(error) => {
                respond(&IrResponse { ir: None, diagnostics: vec![to_wire(&json_diag(error.to_string()))] })
            }
        },
        Err(error) => respond(&IrResponse {
            ir: None,
            diagnostics: vec![to_wire(&dejavu_engine::parse_error_diagnostic(error))],
        }),
    }
}

/// 规范化 IR JSON（丢弃 span / 空文本等不影响语义的形状）。
#[wasm_bindgen(js_name = "normalizeIr")]
pub fn normalize_ir(request_json: &str) -> String {
    let request: NormalizeRequest = match from_request(request_json) {
        Ok(request) => request,
        Err(diagnostic) => return respond(&IrResponse { ir: None, diagnostics: vec![diagnostic] }),
    };
    match dejavu_ir::normalize_ir_json(&request.ir) {
        Ok(value) => match serde_json::to_string(&value) {
            Ok(ir) => respond(&IrResponse { ir: Some(ir), diagnostics: Vec::new() }),
            Err(error) => failure_ir(json_diag(error.to_string())),
        },
        Err(error) => match error {
            IrError::Diagnostic(diagnostic) => {
                respond(&IrResponse { ir: None, diagnostics: vec![to_wire(&diagnostic)] })
            }
            IrError::Json(error) => failure_ir(json_diag(error.to_string())),
        },
    }
}

/// 渲染 IR JSON。
#[wasm_bindgen(js_name = "renderIr")]
pub fn render_ir_api(request_json: &str) -> String {
    let request: RenderRequest = match from_request(request_json) {
        Ok(request) => request,
        Err(diagnostic) => return respond(&RenderResponse { output: None, diagnostics: vec![diagnostic] }),
    };
    let document = match serde_json::from_str::<IrDocument>(&request.ir) {
        Ok(document) => document,
        Err(error) => {
            return respond(&RenderResponse { output: None, diagnostics: vec![to_wire(&json_diag(error.to_string()))] });
        }
    };
    render_document(&document, &request.context, request.options.as_deref(), request.catalog.as_deref())
}

/// 解析并渲染模板源码。
#[wasm_bindgen(js_name = "renderSource")]
pub fn render_source_api(request_json: &str) -> String {
    let request: RenderSourceRequest = match from_request(request_json) {
        Ok(request) => request,
        Err(diagnostic) => return respond(&RenderResponse { output: None, diagnostics: vec![diagnostic] }),
    };
    let language = request
        .delimiters
        .as_deref()
        .and_then(|s| serde_json::from_str::<dejavu_ir::Language>(s).ok())
        .unwrap_or_else(dejavu_ir::default_language);
    let file = request
        .options
        .as_deref()
        .and_then(|raw| serde_json::from_str::<RenderOptionsWire>(raw).ok())
        .and_then(|wire| wire.name)
        .unwrap_or_else(|| "template.dejavu".into());
    let document = match dejavu_language::parse_with(&request.source, &file, &language) {
        Ok(document) => document,
        Err(error) => {
            return respond(&RenderResponse {
                output: None,
                diagnostics: vec![to_wire(&dejavu_engine::parse_error_diagnostic(error))],
            });
        }
    };
    render_document(&document, &request.context, request.options.as_deref(), request.catalog.as_deref())
}

/// 纯 path-v1 引用解析：不读文件、不解析模板。
#[wasm_bindgen(js_name = "resolveTemplateRef")]
pub fn resolve_template_ref(request_json: &str) -> String {
    let request: ResolveRequest = match from_request(request_json) {
        Ok(request) => request,
        Err(diagnostic) => return respond(&ResolveResponse { id: None, diagnostics: vec![diagnostic] }),
    };
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
    let extensions = request.extensions.unwrap_or_else(|| {
        DEFAULT_EXTENSIONS.iter().map(|s| s.to_string()).collect()
    });

    match resolve_ref(&roots, &extensions, &request.reference, request.from.as_deref()) {
        Ok(hit) => respond(&ResolveResponse {
            id: roots.get(hit.root_index).map(|root| dejavu_ir::canonical_id(&root.name, &hit.path)),
            diagnostics: Vec::new(),
        }),
        Err(diagnostic) => respond(&ResolveResponse { id: None, diagnostics: vec![to_wire(&diagnostic)] }),
    }
}

/// 将单模板 IR 预编译为 ESM JavaScript 模块。
#[wasm_bindgen(js_name = "precompileIrSingle")]
pub fn precompile_ir_single(request_json: &str) -> String {
    let request: PrecompileSingleRequest = match from_request(request_json) {
        Ok(request) => request,
        Err(diagnostic) => return respond(&PrecompileSingleResponse {
            module: None,
            diagnostics: vec![diagnostic],
            entry_id: String::new(),
        }),
    };
    match dejavu_codegen::precompile_single(&request.ir, request.options.as_deref()) {
        Ok(result) => respond(&PrecompileSingleResponse {
            module: Some(GeneratedModuleWire {
                path: "index.js".to_string(),
                code: result.code,
                dependencies: result.dependencies,
                origins: to_wire_origins(&result.origins),
            }),
            diagnostics: result.diagnostics.iter().map(to_wire).collect(),
            entry_id: result.entry_id,
        }),
        Err(err) => respond(&PrecompileSingleResponse {
            module: None,
            diagnostics: vec![to_wire(
                &Diagnostic::new(Severity::Error, codes::IR_INVALID_JSON, Stage::Compile).message(err.to_string()),
            )],
            entry_id: String::new(),
        }),
    }
}

/// 将模板 catalog 预编译为多模块 ESM AOT bundle。
#[wasm_bindgen(js_name = "precompileCatalog")]
pub fn precompile_catalog_api(request_json: &str) -> String {
    let request: PrecompileCatalogRequest = match from_request(request_json) {
        Ok(request) => request,
        Err(diagnostic) => return respond(&PrecompileCatalogResponse {
            modules: Vec::new(),
            manifest: None,
            diagnostics: vec![diagnostic],
            entry_id: String::new(),
        }),
    };
    match dejavu_codegen::precompile_catalog(&request.entry_ir, &request.catalog, request.options.as_deref()) {
        Ok(result) => respond(&PrecompileCatalogResponse {
            modules: result
                .modules
                .into_iter()
                .map(|m| GeneratedModuleWire {
                    path: m.path,
                    code: m.code,
                    dependencies: m.dependencies,
                    origins: to_wire_origins(&m.origins),
                })
                .collect(),
            manifest: Some(result.manifest),
            diagnostics: result.diagnostics.iter().map(to_wire).collect(),
            entry_id: result.entry_id,
        }),
        Err(err) => respond(&PrecompileCatalogResponse {
            modules: Vec::new(),
            manifest: None,
            diagnostics: vec![to_wire(
                &Diagnostic::new(Severity::Error, codes::IR_INVALID_JSON, Stage::Compile).message(err.to_string()),
            )],
            entry_id: String::new(),
        }),
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

fn render_document(document: &IrDocument, context: &str, options: Option<&str>, catalog: Option<&str>) -> String {
    let context: Value = match serde_json::from_str(context) {
        Ok(value) => value,
        Err(error) => {
            return respond(&RenderResponse { output: None, diagnostics: vec![to_wire(&json_diag(error.to_string()))] });
        }
    };

    let wire = match options {
        Some(raw) => match serde_json::from_str::<RenderOptionsWire>(raw) {
            Ok(wire) => wire,
            Err(error) => {
                return respond(&RenderResponse { output: None, diagnostics: vec![to_wire(&json_diag(error.to_string()))] });
            }
        },
        None => RenderOptionsWire::default(),
    };

    let mut catalog_state = None;
    if let Some(raw) = catalog {
        match build_catalog(raw) {
            Ok(value) => catalog_state = Some(value),
            Err(diagnostic) => {
                return respond(&RenderResponse { output: None, diagnostics: vec![to_wire(&diagnostic)] });
            }
        }
    }

    let options = RenderOptions { strict_undefined: wire.strict, name: wire.name };
    let mut loader_state = catalog_state.as_ref().map(dejavu_ir::CatalogLoader::new);
    let loader: Option<&mut dyn TemplateLoader> = match loader_state.as_mut() {
        Some(loader) => Some(loader),
        None => None,
    };
    match render_ir_opts(document, &context, options, loader) {
        Ok(output) => respond(&RenderResponse { output: Some(output), diagnostics: Vec::new() }),
        Err(error) => respond(&RenderResponse { output: None, diagnostics: vec![to_wire(&ir_error_diagnostic(error))] }),
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

    let extensions = spec.extensions.unwrap_or_else(|| {
        DEFAULT_EXTENSIONS.iter().map(|s| s.to_string()).collect()
    });
    Ok(TemplateCatalog::new(roots, extensions))
}

fn failure_ir(diagnostic: Diagnostic) -> String {
    respond(&IrResponse { ir: None, diagnostics: vec![to_wire(&diagnostic)] })
}

fn json_diag(message: String) -> Diagnostic {
    Diagnostic::new(Severity::Error, codes::IR_INVALID_JSON, Stage::Host).message(message)
}

fn ir_error_diagnostic(error: IrError) -> Diagnostic {
    match error {
        IrError::Diagnostic(diagnostic) => diagnostic,
        IrError::Json(error) => {
            Diagnostic::new(Severity::Error, codes::IR_INVALID_JSON, Stage::Host).message(error.to_string())
        }
    }
}

fn to_wire_origins(origins: &[dejavu_codegen::LineOrigin]) -> Vec<GeneratedOriginWire> {
    origins
        .iter()
        .map(|origin| GeneratedOriginWire { line: origin.line, start: origin.start, end: origin.end })
        .collect()
}

fn to_wire(diagnostic: &Diagnostic) -> DiagnosticWire {
    DiagnosticWire {
        code: diagnostic.code.clone(),
        severity: severity_name(diagnostic.severity).to_string(),
        stage: stage_name(diagnostic.stage).to_string(),
        message: diagnostic.message.clone().unwrap_or_default(),
        path: diagnostic.path.clone(),
        template_id: diagnostic.template_id.clone(),
        span: diagnostic.span.map(|span| SpanWire { start: span.start as u32, end: span.end as u32 }),
        args_json: serde_json::to_string(&diagnostic.args).unwrap_or_else(|_| "{}".to_string()),
        related: diagnostic
            .related
            .iter()
            .map(|related| RelatedSpanWire {
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

/// 请求 JSON 解析失败：不 panic，返回 host 阶段的结构化诊断。
fn from_request<T: serde::de::DeserializeOwned>(raw: &str) -> Result<T, DiagnosticWire> {
    serde_json::from_str(raw).map_err(|error| to_wire(&json_diag(format!("invalid request json: {error}"))))
}

fn respond<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| r#"{"diagnostics":[]}"#.to_string())
}
