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
use dejavu_ir::registry::{CapabilitySet, CustomEntry, Namespace};
use dejavu_ir::{
    CatalogRoot, CustomError, CustomRegistry, IrDocument, IrError, RenderOptions, ResolveRoot, TemplateCatalog,
    TemplateLoader, render_ir_opts, resolve_ref,
};
use js_sys::{JSON, Reflect};
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
///
/// `registry` 是可选的 JS 注册表对象（`{ entries, filter, function, test }`），
/// 缺省 / `undefined` / `null` 时只用内置注册表。
#[wasm_bindgen(js_name = "renderIr")]
pub fn render_ir_api(request_json: &str, registry: Option<JsValue>) -> String {
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
    let registry = match JsCustomRegistry::from_value(registry) {
        Ok(registry) => registry,
        Err(diagnostic) => {
            return respond(&RenderResponse { output: None, diagnostics: vec![to_wire(&diagnostic)] });
        }
    };
    render_document(&document, &request.context, request.options.as_deref(), request.catalog.as_deref(), registry.as_ref())
}

/// 解析并渲染模板源码。
#[wasm_bindgen(js_name = "renderSource")]
pub fn render_source_api(request_json: &str, registry: Option<JsValue>) -> String {
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
    let registry = match JsCustomRegistry::from_value(registry) {
        Ok(registry) => registry,
        Err(diagnostic) => {
            return respond(&RenderResponse { output: None, diagnostics: vec![to_wire(&diagnostic)] });
        }
    };
    render_document(&document, &request.context, request.options.as_deref(), request.catalog.as_deref(), registry.as_ref())
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
    #[serde(default)]
    capabilities: Option<Vec<String>>,
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

fn render_document(
    document: &IrDocument,
    context: &str,
    options: Option<&str>,
    catalog: Option<&str>,
    custom: Option<&JsCustomRegistry>,
) -> String {
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

    let capabilities = match wire.capabilities {
        Some(names) => match CapabilitySet::from_names(&names) {
            Ok(set) => set,
            Err(name) => {
                return respond(&RenderResponse { output: None, diagnostics: vec![to_wire(&capability_diag(&name))] });
            }
        },
        None => CapabilitySet::all(),
    };
    let custom: Option<&dyn CustomRegistry> = custom.map(|registry| registry as &dyn CustomRegistry);
    let options = RenderOptions { strict_undefined: wire.strict, name: wire.name, capabilities, custom };
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

// ---------------------------------------------------------------------------
// JS 自定义注册表
// ---------------------------------------------------------------------------

/// Host-supplied registry passed as a JS object to `renderIr` / `renderSource`.
///
/// Shape: `{ entries: CustomEntry[], filter?: {name: fn}, function?: {name: fn},
/// test?: {name: fn} }`. Descriptions use the same `CustomEntry` wire as the
/// Rust and AOT paths, so the conflict / arity / capability rules stay in one
/// place. Implementations are synchronous and only exchange JSON-friendly
/// values. Any exception becomes `dejavu.template.custom-invocation` with the
/// exception message, because the renderer trait itself never unwinds.
struct JsCustomRegistry {
    entries: Vec<CustomEntry>,
    filter: Option<JsValue>,
    function: Option<JsValue>,
    test: Option<JsValue>,
}

impl JsCustomRegistry {
    fn from_value(value: Option<JsValue>) -> Result<Option<Self>, Diagnostic> {
        let Some(value) = value else { return Ok(None) };
        if value.is_undefined() || value.is_null() {
            return Ok(None);
        }
        let entries_value = get_property(&value, "entries")?;
        let entries: Vec<CustomEntry> = if entries_value.is_undefined() || entries_value.is_null() {
            Vec::new()
        } else {
            let text = JSON::stringify(&entries_value)
                .map_err(|error| json_diag(js_error_message(&error)))?;
            serde_json::from_str(&String::from(text))
                .map_err(|error| json_diag(format!("invalid registry entries: {error}")))?
        };
        Ok(Some(Self {
            entries,
            filter: optional_namespace(&value, "filter")?,
            function: optional_namespace(&value, "function")?,
            test: optional_namespace(&value, "test")?,
        }))
    }

    fn invoke(
        &self,
        namespace: Namespace,
        name: &str,
        arguments: &js_sys::Array,
    ) -> Result<JsValue, CustomError> {
        let object = match namespace {
            Namespace::Filter => self.filter.as_ref(),
            Namespace::Function => self.function.as_ref(),
            Namespace::Test => self.test.as_ref(),
        };
        let Some(object) = object else {
            return Err(CustomError::new(format!(
                "host registry has no `{}` implementations",
                namespace.as_str()
            )));
        };
        let implementation = Reflect::get(object, &JsValue::from_str(name))
            .map_err(|error| CustomError::new(js_error_message(&error)))?;
        if !implementation.is_function() {
            return Err(CustomError::new(format!("host registry has no `{name}` implementation")));
        }
        let function: js_sys::Function = implementation.unchecked_into();
        Reflect::apply(&function, object, arguments)
            .map_err(|error| CustomError::new(js_error_message(&error)))
    }
}

impl CustomRegistry for JsCustomRegistry {
    fn entries(&self) -> &[CustomEntry] {
        &self.entries
    }

    fn apply_filter(&self, name: &str, value: &Value, args: &[Value]) -> Result<Value, CustomError> {
        let arguments = js_sys::Array::new();
        arguments.push(&value_to_js(value)?);
        arguments.push(&values_to_js(args)?);
        js_to_value(self.invoke(Namespace::Filter, name, &arguments)?)
    }

    fn call_function(&self, name: &str, args: &[Value]) -> Result<Value, CustomError> {
        let arguments = js_sys::Array::new();
        arguments.push(&values_to_js(args)?);
        js_to_value(self.invoke(Namespace::Function, name, &arguments)?)
    }

    fn apply_test(&self, name: &str, value: &Value, args: &[Value]) -> Result<bool, CustomError> {
        let arguments = js_sys::Array::new();
        arguments.push(&value_to_js(value)?);
        arguments.push(&values_to_js(args)?);
        Ok(self.invoke(Namespace::Test, name, &arguments)?.as_bool().unwrap_or(false))
    }
}

fn get_property(object: &JsValue, key: &str) -> Result<JsValue, Diagnostic> {
    Reflect::get(object, &JsValue::from_str(key)).map_err(|error| json_diag(js_error_message(&error)))
}

fn optional_namespace(object: &JsValue, key: &str) -> Result<Option<JsValue>, Diagnostic> {
    let value = get_property(object, key)?;
    Ok(if value.is_undefined() || value.is_null() { None } else { Some(value) })
}

fn values_to_js(values: &[Value]) -> Result<JsValue, CustomError> {
    let array = js_sys::Array::new();
    for value in values {
        array.push(&value_to_js(value)?);
    }
    Ok(array.into())
}

fn value_to_js(value: &Value) -> Result<JsValue, CustomError> {
    let text = serde_json::to_string(value).map_err(|error| CustomError::new(error.to_string()))?;
    JSON::parse(&text).map_err(|error| CustomError::new(js_error_message(&error)))
}

fn js_to_value(value: JsValue) -> Result<Value, CustomError> {
    if value.is_undefined() || value.is_null() {
        return Ok(Value::Null);
    }
    let text = JSON::stringify(&value).map_err(|error| CustomError::new(js_error_message(&error)))?;
    serde_json::from_str(&String::from(text)).map_err(|error| CustomError::new(error.to_string()))
}

fn js_error_message(error: &JsValue) -> String {
    error
        .as_string()
        .or_else(|| Reflect::get(error, &JsValue::from_str("message")).ok().and_then(|value| value.as_string()))
        .unwrap_or_else(|| "host registry implementation failed".to_string())
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

fn capability_diag(name: &str) -> Diagnostic {
    Diagnostic::new(Severity::Error, codes::UNKNOWN_CAPABILITY, Stage::Host)
        .message(format!("unknown capability `{name}`"))
        .arg("capability", name)
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
