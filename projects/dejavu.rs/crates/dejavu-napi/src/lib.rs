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
use dejavu_ir::registry::{CapabilitySet, CustomEntry, Namespace};
use dejavu_ir::{
    CatalogRoot, CustomError, CustomRegistry, IrDocument, IrError, RenderOptions, ResolveRoot, TemplateCatalog,
    TemplateLoader, render_ir_opts, resolve_ref,
};
use napi::bindgen_prelude::{Env, FnArgs, Function, Object, Unknown};
use napi_derive::napi;
use serde::Deserialize;
use serde_json::Value;

/// 绑定合同标识。
pub const BINDING_CONTRACT: &str = "dejavu.napi/3";

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
    /// 可选的 Language JSON，覆盖默认 delimiters（`<% %>` / `<# #>`）。
    pub delimiters: Option<String>,
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
    /// 可选的 Language JSON，覆盖默认 delimiters。
    pub delimiters: Option<String>,
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

/// `precompileIrSingle` 请求。
#[napi(object)]
pub struct PrecompileSingleRequest {
    /// 入口模板的 IR JSON。
    pub ir: String,
    /// `{ strictUndefined?: bool, entryId?: string }` JSON。
    pub options: Option<String>,
}

/// 单/多模块 AOT codegen 产出的模块描述（N-API wire）。
#[napi(object)]
pub struct GeneratedModule {
    /// 相对输出根的模块路径，例如 `index.js` 或 `_internal/partial__footer.js`。
    pub path: String,
    /// ESM JS 源码。
    pub code: String,
    /// 本模块 import 的 canonical template id 列表（用于多模板场景追踪依赖）。
    pub dependencies: Vec<String>,
    /// 生成行 → 模板 span 的映射点，宿主据此生成 source map。
    pub origins: Vec<GeneratedOrigin>,
}

/// 一个生成行起点及其模板 span。
#[napi(object)]
pub struct GeneratedOrigin {
    /// 模块内 0-based 生成行号。
    pub line: u32,
    /// 模板字节偏移起点。
    pub start: u32,
    /// 模板字节偏移终点（exclusive）。
    pub end: u32,
}

fn to_js_origins(origins: &[dejavu_codegen::LineOrigin]) -> Vec<GeneratedOrigin> {
    origins
        .iter()
        .map(|origin| GeneratedOrigin {
            line: origin.line,
            start: origin.start,
            end: origin.end,
        })
        .collect()
}

/// 单模板 AOT 响应。
#[napi(object)]
pub struct PrecompileSingleResponse {
    /// 生成的模块（单模板就是一个模块）。
    pub module: Option<GeneratedModule>,
    /// 全部诊断。
    pub diagnostics: Vec<JsDiagnostic>,
    /// 入口 template id。
    pub entry_id: String,
}

/// `precompileCatalog` 请求。
#[napi(object)]
pub struct PrecompileCatalogRequest {
    /// 入口模板的 IR JSON。
    pub entry_ir: String,
    /// Catalog 规格 JSON（`{ roots: [{name, files: {path: ir_json}}], extensions? }`）。
    pub catalog: String,
    /// `{ strictUndefined?: bool, entryId?: string }` JSON。
    pub options: Option<String>,
}

/// Catalog AOT 响应。
#[napi(object)]
pub struct PrecompileCatalogResponse {
    /// 全部生成的模块。
    pub modules: Vec<GeneratedModule>,
    /// 整个 catalog 的 manifest JSON（TS 侧可直接写文件）。
    pub manifest: Option<String>,
    /// 全部诊断。
    pub diagnostics: Vec<JsDiagnostic>,
    /// 入口 template id。
    pub entry_id: String,
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
    let language = request
        .delimiters
        .as_deref()
        .and_then(|s| serde_json::from_str::<dejavu_ir::Language>(s).ok())
        .unwrap_or_else(dejavu_ir::default_language);
    let file = request.file.as_deref().unwrap_or("template.dejavu");
    let parsed = dejavu_language::parse_with(&request.source, file, &language);

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
pub fn render_ir_api(env: Env, request: RenderRequest, registry: Option<Object>) -> RenderResponse {
    let document = match serde_json::from_str::<IrDocument>(&request.ir) {
        Ok(document) => document,
        Err(error) => return RenderResponse { output: None, diagnostics: vec![to_js(&json_diag(error.to_string()))] },
    };
    let registry = match JsCustomRegistry::from_object(&env, registry) {
        Ok(registry) => registry,
        Err(diagnostic) => {
            return RenderResponse { output: None, diagnostics: vec![to_js(&diagnostic)] };
        }
    };
    render_document(
        &document,
        &request.context,
        request.options.as_deref(),
        request.catalog.as_deref(),
        registry.as_ref().map(|registry| registry as &dyn CustomRegistry),
    )
}

/// 解析并渲染模板源码。
#[napi(js_name = "renderSource")]
pub fn render_source_api(env: Env, request: RenderSourceRequest, registry: Option<Object>) -> RenderResponse {
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
    let document = dejavu_language::parse_with(&request.source, &file, &language);
    let document = match document {
        Ok(document) => document,
        Err(error) => {
            return RenderResponse { output: None, diagnostics: vec![to_js(&dejavu_engine::parse_error_diagnostic(error))] };
        }
    };
    let registry = match JsCustomRegistry::from_object(&env, registry) {
        Ok(registry) => registry,
        Err(diagnostic) => {
            return RenderResponse { output: None, diagnostics: vec![to_js(&diagnostic)] };
        }
    };
    render_document(
        &document,
        &request.context,
        request.options.as_deref(),
        request.catalog.as_deref(),
        registry.as_ref().map(|registry| registry as &dyn CustomRegistry),
    )
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

/// 将单模板 IR 预编译为 ESM JavaScript 模块。
#[napi(js_name = "precompileIrSingle")]
pub fn precompile_ir_single_api(request: PrecompileSingleRequest) -> PrecompileSingleResponse {
    match dejavu_codegen::precompile_single(&request.ir, request.options.as_deref()) {
        Ok(result) => PrecompileSingleResponse {
            module: Some(GeneratedModule {
                path: "index.js".to_string(),
                code: result.code,
                dependencies: result.dependencies,
                origins: to_js_origins(&result.origins),
            }),
            diagnostics: result.diagnostics.iter().map(to_js).collect(),
            entry_id: result.entry_id,
        },
        Err(err) => PrecompileSingleResponse {
            module: None,
            diagnostics: vec![to_js(
                &Diagnostic::new(Severity::Error, codes::IR_INVALID_JSON, Stage::Compile)
                    .message(err.to_string()),
            )],
            entry_id: String::new(),
        },
    }
}

/// 将模板 catalog 预编译为多模块 ESM AOT bundle。
#[napi(js_name = "precompileCatalog")]
pub fn precompile_catalog_api(request: PrecompileCatalogRequest) -> PrecompileCatalogResponse {
    match dejavu_codegen::precompile_catalog(&request.entry_ir, &request.catalog, request.options.as_deref()) {
        Ok(result) => PrecompileCatalogResponse {
            modules: result
                .modules
                .into_iter()
                .map(|m| GeneratedModule {
                    path: m.path,
                    code: m.code,
                    dependencies: m.dependencies,
                    origins: to_js_origins(&m.origins),
                })
                .collect(),
            manifest: Some(result.manifest),
            diagnostics: result.diagnostics.iter().map(to_js).collect(),
            entry_id: result.entry_id,
        },
        Err(err) => PrecompileCatalogResponse {
            modules: Vec::new(),
            manifest: None,
            diagnostics: vec![to_js(
                &Diagnostic::new(Severity::Error, codes::IR_INVALID_JSON, Stage::Compile)
                    .message(err.to_string()),
            )],
            entry_id: String::new(),
        },
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

// ---------------------------------------------------------------------------
// JS registry adapter
// ---------------------------------------------------------------------------

/// Host-supplied registry passed as the optional `registry` argument.
///
/// Shape: `{ entries: CustomEntry[], filter?: {name: fn}, function?: {name: fn},
/// test?: {name: fn} }`, identical to the WASM bridge. Implementations run
/// synchronously on the calling JS thread; a thrown exception becomes
/// `dejavu.template.custom-invocation` so the render can still report a
/// structured diagnostic instead of unwinding Node.
struct JsCustomRegistry<'env> {
    env: Env,
    entries: Vec<CustomEntry>,
    filter: Option<Object<'env>>,
    function: Option<Object<'env>>,
    test: Option<Object<'env>>,
}

impl<'env> JsCustomRegistry<'env> {
    fn from_object(env: &Env, registry: Option<Object<'env>>) -> Result<Option<Self>, Diagnostic> {
        let Some(registry) = registry else { return Ok(None) };
        let entries: Vec<CustomEntry> = match registry.get::<Unknown<'env>>("entries").map_err(napi_diag)? {
            Some(value) => env
                .from_js_value::<Vec<CustomEntry>, _>(value)
                .map_err(|error| json_diag(format!("invalid registry entries: {}", error.reason)))?,
            None => Vec::new(),
        };
        Ok(Some(Self {
            env: *env,
            entries,
            filter: registry.get("filter").map_err(napi_diag)?,
            function: registry.get("function").map_err(napi_diag)?,
            test: registry.get("test").map_err(napi_diag)?,
        }))
    }

    fn namespace(&self, namespace: Namespace) -> Result<&Object<'env>, CustomError> {
        let object = match namespace {
            Namespace::Filter => self.filter.as_ref(),
            Namespace::Function => self.function.as_ref(),
            Namespace::Test => self.test.as_ref(),
        };
        object.ok_or_else(|| {
            CustomError::new(format!("host registry has no `{}` implementations", namespace.as_str()))
        })
    }

    fn missing(name: &str) -> CustomError {
        CustomError::new(format!("host registry has no `{name}` implementation"))
    }

    fn values_to_js(&self, values: &[Value]) -> Result<Unknown<'env>, CustomError> {
        let array = Value::Array(values.to_vec());
        self.env.to_js_value(&array).map_err(|error| CustomError::new(error.reason))
    }

    fn js_to_value(&self, value: Unknown<'env>) -> Result<Value, CustomError> {
        self.env.from_js_value::<Value, _>(value).map_err(|error| CustomError::new(error.reason))
    }

    fn call_with_two(
        &self,
        namespace: Namespace,
        name: &str,
        first: Unknown<'env>,
        second: Unknown<'env>,
    ) -> Result<Unknown<'env>, CustomError> {
        let object = self.namespace(namespace)?;
        let function = object
            .get::<Function<'env, FnArgs<(Unknown<'env>, Unknown<'env>)>, Unknown<'env>>>(name)
            .map_err(|error| CustomError::new(error.reason))?
            .ok_or_else(|| Self::missing(name))?;
        function.call(FnArgs::from((first, second))).map_err(|error| CustomError::new(error.reason))
    }

    fn call_with_one(
        &self,
        namespace: Namespace,
        name: &str,
        argument: Unknown<'env>,
    ) -> Result<Unknown<'env>, CustomError> {
        let object = self.namespace(namespace)?;
        let function = object
            .get::<Function<'env, FnArgs<(Unknown<'env>,)>, Unknown<'env>>>(name)
            .map_err(|error| CustomError::new(error.reason))?
            .ok_or_else(|| Self::missing(name))?;
        function.call(FnArgs::from((argument,))).map_err(|error| CustomError::new(error.reason))
    }
}

impl<'env> CustomRegistry for JsCustomRegistry<'env> {
    fn entries(&self) -> &[CustomEntry] {
        &self.entries
    }

    fn apply_filter(&self, name: &str, value: &Value, args: &[Value]) -> Result<Value, CustomError> {
        let first = self.env.to_js_value(value).map_err(|error| CustomError::new(error.reason))?;
        let second = self.values_to_js(args)?;
        self.js_to_value(self.call_with_two(Namespace::Filter, name, first, second)?)
    }

    fn call_function(&self, name: &str, args: &[Value]) -> Result<Value, CustomError> {
        let argument = self.values_to_js(args)?;
        self.js_to_value(self.call_with_one(Namespace::Function, name, argument)?)
    }

    fn apply_test(&self, name: &str, value: &Value, args: &[Value]) -> Result<bool, CustomError> {
        let first = self.env.to_js_value(value).map_err(|error| CustomError::new(error.reason))?;
        let second = self.values_to_js(args)?;
        let result = self.call_with_two(Namespace::Test, name, first, second)?;
        self.env.from_js_value::<bool, _>(result).map_err(|error| CustomError::new(error.reason))
    }
}

fn napi_diag(error: napi::Error) -> Diagnostic {
    json_diag(error.reason)
}

fn render_document(
    document: &IrDocument,
    context: &str,
    options: Option<&str>,
    catalog: Option<&str>,
    custom: Option<&dyn CustomRegistry>,
) -> RenderResponse {
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

    let capabilities = match wire.capabilities {
        Some(names) => match CapabilitySet::from_names(&names) {
            Ok(set) => set,
            Err(name) => {
                return RenderResponse { output: None, diagnostics: vec![to_js(&capability_diag(&name))] };
            }
        },
        None => CapabilitySet::all(),
    };
    let options = RenderOptions { strict_undefined: wire.strict, name: wire.name, capabilities, custom };
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

fn capability_diag(name: &str) -> Diagnostic {
    Diagnostic::new(Severity::Error, codes::UNKNOWN_CAPABILITY, Stage::Host)
        .message(format!("unknown capability `{name}`"))
        .arg("capability", name)
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
