use crate::{Diagnostic, IrError, codes, diag_load, diag_render};
use crate::registry::{
    Capability, CapabilitySet, CustomEntry, FilterLookupError, Namespace, custom_entry, validate_filter,
};
use crate::types::{IrDocument, IrNode, PartialParam};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Maximum nested macro invocations before `dejavu.template.macro-recursion`.
pub const MACRO_RECURSION_LIMIT: usize = 32;

/// Maximum nested `partial` entries before `dejavu.template.partial-recursion`.
///
/// A template declared as `partial <params>` may include itself (directly or
/// through other partials) to render nested data. This is the single source of
/// truth for that bound; AOT codegen inlines the same number.
pub const PARTIAL_RECURSION_LIMIT: usize = 32;

// ---------------------------------------------------------------------------
// RenderOptions：宿主可选渲染参数（严格模式、依赖追踪等）
// ---------------------------------------------------------------------------

/// Why a host-supplied registry implementation failed while being invoked.
///
/// The host owns the failure text (a JS exception message, a native error
/// chain, …). The renderer turns it into
/// [`codes::CUSTOM_INVOCATION`] with `namespace` / `name` / `message` args so
/// every host reports the same code regardless of its native error type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomError {
    /// Host-provided detail; never parsed by the renderer.
    pub message: String,
}

impl CustomError {
    /// Build a failure from a host message.
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

/// Host-supplied registry implementations.
///
/// The host owns both the description list ([`CustomRegistry::entries`]) and the
/// synchronous implementations. Entries are validated once per render
/// (`dejavu.template.registry-conflict`), then dispatched by canonical name so
/// aliases never reach two different implementations. The trait is deliberately
/// synchronous, immutable for the duration of a render and free of any
/// `eval` / source-carrying channel.
pub trait CustomRegistry {
    /// Descriptions of every entry this host supplies.
    fn entries(&self) -> &[CustomEntry];
    /// Apply a `filter` entry to `value` with already-evaluated explicit args.
    fn apply_filter(&self, name: &str, value: &Value, args: &[Value]) -> Result<Value, CustomError>;
    /// Evaluate a `function` entry with already-evaluated arguments.
    fn call_function(&self, name: &str, args: &[Value]) -> Result<Value, CustomError>;
    /// Evaluate a `test` entry against an already-evaluated value.
    fn apply_test(&self, name: &str, value: &Value, args: &[Value]) -> Result<bool, CustomError>;
}

/// 渲染选项 —— 匹配 TS `RenderOptions`。
///
/// 默认 `strict_undefined = false`（缺失变量 → null → 空串），
/// `capabilities` 默认放开全部注册表能力，`custom` 为空（只有内置条目）。
pub struct RenderOptions<'a> {
    /// 开启后，引用未定义变量抛 `UNDEFINED_VARIABLE` 错误。
    pub strict_undefined: bool,
    /// 模板 canonical id（用于 include 循环检测和依赖追踪）。
    pub name: Option<String>,
    /// 宿主授予的注册表能力；超出集合的 filter/function/test 直接拒绝。
    pub capabilities: CapabilitySet,
    /// 宿主提供的自定义注册表实现；注册在渲染前固定，单次渲染内不可变。
    pub custom: Option<&'a dyn CustomRegistry>,
}

impl Default for RenderOptions<'_> {
    fn default() -> Self {
        Self {
            strict_undefined: false,
            name: None,
            capabilities: CapabilitySet::all(),
            custom: None,
        }
    }
}

// ---------------------------------------------------------------------------
// 公共入口
// ---------------------------------------------------------------------------

/// 模板加载器：根据 ref + 来源解析到 canonical id + IR。
///
/// path-v1 合同：`resolve(ref, from?)` 返回 canonical id + 文档。
/// 简单宿主可用原始 path 作 id — 只要 `resolve` 把同一引用稳定映射到同一 id，
/// 就足以让 renderer 在 canonical id 层面做循环检测。
pub trait TemplateLoader {
    /// 解析 ref → canonical id + IrDocument。
    ///
    /// `from` 是调用方的 canonical id（include/extends 引用时的来源模板），
    /// 相对路径解析会用到它。`from` 可以为空（顶层渲染）。
    fn resolve(&mut self, ref_: &str, from: Option<&str>) -> Result<ResolvedTemplate, IrError>;
}

/// resolve 返回的模板：canonical id + 已加载文档。
#[derive(Debug, Clone)]
pub struct ResolvedTemplate {
    pub id: String,
    pub document: IrDocument,
}

/// 渲染一个 IR 文档。无 loader 时 `extends`/`include`/`super` 会报错。
///
/// 这是现有 T1 路径的向后兼容入口。需要完整模板继承链时用 [`render_ir_with_loader`]。
pub fn render_ir(doc: &IrDocument, ctx: &Value) -> Result<String, IrError> {
    render_ir_opts(doc, ctx, RenderOptions::default(), None)
}

/// 渲染一个 IR 文档，带完整模板支持：`extends`、`include`、`block`、`super`。
pub fn render_ir_with_loader(
    doc: &IrDocument,
    ctx: &Value,
    loader: &mut dyn TemplateLoader,
) -> Result<String, IrError> {
    render_ir_opts(doc, ctx, RenderOptions::default(), Some(loader))
}

/// 底层入口：带 RenderOptions + 可选 loader。
pub fn render_ir_opts<'a>(
    doc: &IrDocument,
    ctx: &Value,
    opts: RenderOptions<'a>,
    loader: Option<&'a mut dyn TemplateLoader>,
) -> Result<String, IrError> {
    let scope = json_object_to_map(ctx)?;
    if let Some(custom) = opts.custom {
        crate::registry::validate_custom_entries(custom.entries())?;
    }
    let default_id = opts.name.clone().unwrap_or_else(|| "<main>".into());
    let mut r = Renderer {
        scope,
        loader,
        custom: opts.custom,
        strict_undefined: opts.strict_undefined,
        capabilities: opts.capabilities,
        current_id: default_id,
        block_levels: Vec::new(),
        block_level: 0,
        current_block: None,
        macro_modules: HashMap::new(),
        macro_depth: 0,
        caller_stack: Vec::new(),
        visiting_extends: HashSet::new(),
        include_stack: Vec::new(),
        partial_depth: 0,
        resolved: HashMap::new(),
    };
    r.render_doc(doc)
}

// ---------------------------------------------------------------------------
// 内部 Renderer
// ---------------------------------------------------------------------------

struct Renderer<'a> {
    scope: HashMap<String, Value>,
    loader: Option<&'a mut dyn TemplateLoader>,
    /// Host-supplied registry implementations, fixed for this render.
    custom: Option<&'a dyn CustomRegistry>,
    strict_undefined: bool,
    /// Host-granted registry capabilities; checked on every filter/function/test.
    capabilities: CapabilitySet,
    current_id: String,
    /// Block definitions per inheritance level. Index 0 is the most derived
    /// template, each further index is one `extends` hop closer to the root.
    block_levels: Vec<HashMap<String, Vec<IrNode>>>,
    /// Which level's block body is being rendered (`super` looks one level up).
    block_level: usize,
    current_block: Option<String>,
    /// Macros declared by each loaded template, keyed by canonical id.
    macro_modules: HashMap<String, HashMap<String, MacroDef>>,
    /// Current macro call nesting depth.
    macro_depth: usize,
    /// Caller blocks supplied by `call`, innermost last. Each block renders in
    /// the scope and module of the template that wrote it.
    caller_stack: Vec<CallerBlock>,
    /// extends 访问过的 canonical id（防止继承环）
    visiting_extends: HashSet<String>,
    /// include 递归栈（防止非 partial 的 include 环）
    include_stack: Vec<String>,
    /// 当前已经进入的 partial 层数（`dejavu.template.partial-recursion` 依据）。
    partial_depth: usize,
    /// 本次渲染内 `(ref, from)` → 已解析模板。
    ///
    /// Key 包含调用方 canonical id，因为相对路径解析依赖它。缓存只在单次
    /// `render_ir_opts` 内有效，跨调用不复用；文档以 `Rc` 共享，重复 include
    /// 不再重复克隆 IR。
    resolved: HashMap<(String, Option<String>), Result<Resolved, Diagnostic>>,
}

/// 缓存中的已解析模板：canonical id + 共享文档。
#[derive(Debug, Clone)]
struct Resolved {
    id: String,
    document: Rc<IrDocument>,
}

/// One template-local macro: name, declared parameters, and body.
///
/// `module` is the canonical id of the template that declared it. Macro
/// bodies resolve names against that module's macro table, so an included
/// template can never see the caller's macros.
#[derive(Debug, Clone)]
struct MacroDef {
    name: String,
    params: Vec<PartialParam>,
    body: Vec<IrNode>,
    module: String,
}

/// Caller block captured by `call`: the body, the lexical scope and module it
/// was written in, and its declared parameters. `caller()` renders it back in
/// that scope with the arguments bound to the parameters.
#[derive(Debug, Clone)]
struct CallerBlock {
    body: Vec<IrNode>,
    scope: HashMap<String, Value>,
    module: String,
    params: Vec<String>,
}

impl<'a> Renderer<'a> {
    /// Resolve a template through the loader, memoized for this render.
    ///
    /// The key is `(ref, from)` because relative resolution depends on the
    /// caller. Both successes and structured failures are cached, so a repeated
    /// failing include reproduces the same diagnostic without re-resolving.
    fn resolve_template(
        &mut self,
        path: &str,
        from: &str,
        feature: &str,
    ) -> Result<Resolved, IrError> {
        let key = (path.to_string(), Some(from.to_string()));
        if let Some(cached) = self.resolved.get(&key) {
            return match cached {
                Ok(resolved) => Ok(resolved.clone()),
                Err(diagnostic) => Err(IrError::Diagnostic(diagnostic.clone())),
            };
        }
        let result = match self.loader.as_deref_mut() {
            Some(loader) => loader.resolve(path, Some(from)),
            None => Err(diag_load(codes::MISSING_LOADER).arg("feature", feature)),
        };
        match result {
            Ok(template) => {
                let resolved = Resolved { id: template.id, document: Rc::new(template.document) };
                self.resolved.insert(key, Ok(resolved.clone()));
                Ok(resolved)
            }
            Err(IrError::Diagnostic(diagnostic)) => {
                self.resolved.insert(key, Err(diagnostic.clone()));
                Err(IrError::Diagnostic(diagnostic))
            }
            Err(other) => Err(other),
        }
    }

    /// Register the macros declared at the top level of the current template.
    ///
    /// A macro whose name matches a registered function is rejected: calls
    /// must have exactly one meaning across JIT and AOT.
    fn register_module(&mut self, doc: &IrDocument) -> Result<(), IrError> {
        let id = self.current_id.clone();
        self.register_document(&id, doc)
    }

    /// Register one template's macros, then resolve its `from ... import`
    /// declarations.
    ///
    /// The local table is inserted before imports are processed so mutual
    /// imports resolve against already-declared macros instead of looping.
    fn register_document(&mut self, id: &str, doc: &IrDocument) -> Result<(), IrError> {
        if self.macro_modules.contains_key(id) {
            return Ok(());
        }
        let IrNode::Template { children, .. } = &doc.body else {
            return Ok(());
        };
        let mut table: HashMap<String, MacroDef> = HashMap::new();
        for child in children {
            if let IrNode::StmtMacro { name, params, body, .. } = child {
                if crate::registry::function_spec(name).is_some() {
                    return Err(diag_render(codes::MACRO_CONFLICT).arg("name", name.clone()));
                }
                table.insert(
                    name.clone(),
                    MacroDef { name: name.clone(), params: params.clone(), body: body.clone(), module: id.to_string() },
                );
            }
        }
        self.macro_modules.insert(id.to_string(), table);

        let mut imported: Vec<(String, MacroDef)> = Vec::new();
        for child in children {
            let IrNode::StmtImport { path, names, .. } = child else {
                continue;
            };
            let path_str = match path.as_ref() {
                IrNode::ExprLiteral { value: Value::String(s), .. } => s.clone(),
                _ => return Err(diag_render(codes::IMPORT_PATH_NOT_LITERAL)),
            };
            let target = self.resolve_template(&path_str, id, "import")?;
            self.register_document(&target.id, &target.document)?;
            let target_table = self.macro_modules.get(&target.id).cloned().unwrap_or_default();
            for spec in names {
                let Some(def) = target_table.get(&spec.name) else {
                    return Err(diag_load(codes::UNKNOWN_IMPORT)
                        .arg("path", path_str.clone())
                        .arg("name", spec.name.clone()));
                };
                let local = spec.local().to_string();
                if self.macro_modules.get(id).is_some_and(|t| t.contains_key(&local)) {
                    return Err(diag_render(codes::MACRO_CONFLICT).arg("name", local));
                }
                let mut def = def.clone();
                def.name = local.clone();
                imported.push((local, def));
            }
        }
        if let Some(table) = self.macro_modules.get_mut(id) {
            for (local, def) in imported {
                table.insert(local, def);
            }
        }
        Ok(())
    }

    fn lookup_macro(&self, name: &str) -> Option<MacroDef> {
        self.macro_modules.get(&self.current_id).and_then(|table| table.get(name)).cloned()
    }

    /// Is this expression a macro invocation (whose output is safe HTML)?
    fn is_macro_call(&self, expr: &IrNode) -> bool {
        if let IrNode::ExprCall { callee, .. } = expr {
            if let IrNode::ExprIdentifier { name, .. } = callee.as_ref() {
                if name == "caller" {
                    return true;
                }
                return self.macro_modules.get(&self.current_id).is_some_and(|t| t.contains_key(name));
            }
        }
        false
    }

    /// Is the outermost filter of this pipe `safe-output`?
    ///
    /// Only the final (outermost) filter decides escaping: `x |> safe` skips
    /// escaping, while `x |> safe |> uppercase` escapes the result. Custom
    /// entries take part in exactly the same rule.
    fn is_raw_filter(&self, expr: &IrNode) -> bool {
        let IrNode::ExprPipe { filter, .. } = expr else {
            return false;
        };
        if let Some(spec) = crate::registry::filter_spec(filter) {
            return spec.capability == Capability::SafeOutput;
        }
        self.custom.is_some_and(|registry| {
            custom_entry(registry.entries(), Namespace::Filter, filter)
                .is_some_and(|entry| entry.capability == Capability::SafeOutput)
        })
    }

    /// Render the innermost `caller()` block in the scope it was written in.
    ///
    /// The block is temporarily removed from the stack so a `caller()` inside
    /// the block cannot recurse into itself, then restored so a macro may
    /// invoke `caller()` more than once.
    fn render_caller_block(&mut self, args: &[Value]) -> Result<Value, IrError> {
        let Some(block) = self.caller_stack.pop() else {
            return Err(diag_render(codes::CALLER_UNAVAILABLE));
        };
        if args.len() != block.params.len() {
            let expected = block.params.len();
            self.caller_stack.push(block);
            return Err(diag_render(codes::CALLER_ARITY)
                .arg("got", args.len())
                .arg("maxArgs", expected));
        }
        let saved_scope = std::mem::take(&mut self.scope);
        let saved_id = std::mem::replace(&mut self.current_id, block.module.clone());
        self.scope = block.scope.clone();
        for (param, value) in block.params.iter().zip(args) {
            self.scope.insert(param.clone(), value.clone());
        }
        let rendered = self.render_list(&block.body).map(Value::String);
        self.scope = saved_scope;
        self.current_id = saved_id;
        self.caller_stack.push(block);
        rendered
    }

    /// Invoke a macro. The body sees its bound parameters only; the caller
    /// context is never inherited.
    fn call_macro(
        &mut self,
        mac: &MacroDef,
        args: &[Value],
        caller: Option<CallerBlock>,
    ) -> Result<Value, IrError> {
        if args.len() > mac.params.len() {
            return Err(diag_render(codes::MACRO_ARITY)
                .arg("macro", mac.name.clone())
                .arg("minArgs", mac.params.iter().filter(|p| p.default.is_none()).count())
                .arg("maxArgs", mac.params.len())
                .arg("got", args.len()));
        }
        for (i, param) in mac.params.iter().enumerate() {
            if i >= args.len() && param.default.is_none() {
                return Err(diag_render(codes::MISSING_MACRO_ARGUMENT)
                    .arg("macro", mac.name.clone())
                    .arg("name", param.name.clone()));
            }
        }
        if self.macro_depth >= MACRO_RECURSION_LIMIT {
            return Err(diag_render(codes::MACRO_RECURSION)
                .arg("macro", mac.name.clone())
                .arg("limit", MACRO_RECURSION_LIMIT));
        }

        let mut scope = HashMap::new();
        for (i, param) in mac.params.iter().enumerate() {
            let value = match args.get(i) {
                Some(v) => v.clone(),
                None => match &param.default {
                    Some(IrNode::ExprLiteral { value, .. }) => value.clone(),
                    _ => Value::Null,
                },
            };
            scope.insert(param.name.clone(), value);
        }

        let saved_scope = std::mem::take(&mut self.scope);
        let saved_id = std::mem::replace(&mut self.current_id, mac.module.clone());
        self.scope = scope;
        self.macro_depth += 1;
        let has_caller = caller.is_some();
        if let Some(block) = caller {
            self.caller_stack.push(block);
        }
        let rendered = self.render_list(&mac.body).map(Value::String);
        if has_caller {
            self.caller_stack.pop();
        }
        self.macro_depth -= 1;
        self.scope = saved_scope;
        self.current_id = saved_id;
        rendered
    }

    fn render_doc(&mut self, doc: &IrDocument) -> Result<String, IrError> {
        // 特殊处理 Template body：先收集 block 定义，处理 extends，再渲染。
        if let IrNode::Template { children, .. } = &doc.body {
            self.register_module(doc)?;
            let mut extends_path: Option<String> = None;
            let mut own_blocks: HashMap<String, Vec<IrNode>> = HashMap::new();

            for c in children {
                match c {
                    IrNode::StmtExtends { parent, .. } => {
                        let Some(parent_str) = literal_path(parent) else {
                            return Err(diag_load(codes::EXTENDS_PATH_NOT_LITERAL)
                                .template_id(self.current_id.clone()));
                        };
                        extends_path = Some(parent_str.to_string());
                    }
                    IrNode::StmtBlock { name, body, .. } => {
                        own_blocks.entry(name.clone()).or_insert_with(|| body.clone());
                    }
                    _ => {}
                }
            }

            // 每个模板都贡献一个继承层级：index 0 是最派生的模板，越往后越靠近根。
            self.block_levels.push(own_blocks);

            if let Some(parent_str) = extends_path {
                let from = self.current_id.clone();
                let parent = self.resolve_template(&parent_str, &from, "extends")?;

                // canonical id 层面的循环检测
                if self.visiting_extends.contains(&parent.id) {
                    return Err(diag_load(codes::CIRCULAR_EXTENDS).arg("chain", parent.id.clone()));
                }
                self.visiting_extends.insert(parent.id.clone());

                let saved_id = std::mem::take(&mut self.current_id);
                self.current_id = parent.id.clone();

                let out = self.render_doc(&parent.document)?;

                self.visiting_extends.remove(&parent.id);
                self.current_id = saved_id;

                return Ok(out);
            }
        }
        // 没有 extends，正常渲染
        self.render_nodes(&doc.body)
    }

    /// Render an included template.
    ///
    /// `with` is the evaluated explicit context and `only` isolates the child
    /// from the caller's context (`only` / `without context`).
    fn render_loaded(
        &mut self,
        path: &str,
        feature: &str,
        with: Option<&Value>,
        only: bool,
    ) -> Result<String, IrError> {
        // Validate the explicit context before mutating renderer state.
        if let Some(Value::Null) | Some(Value::Object(_)) | None = with {
            // null is treated as "no explicit entries"
        } else if let Some(other) = with {
            return Err(diag_render(codes::CONTEXT_NOT_OBJECT).arg("got", json_kind(other)));
        }

        let from = self.current_id.clone();
        let child = self.resolve_template(path, &from, feature)?;

        // A template declared as `partial <params>` binds exactly its declared
        // parameters; it never inherits the caller's context.
        let partial = partial_parts(&child.document.body);

        // A partial may re-enter itself to render nested data. The recursion is
        // bounded so unbounded data fails with one stable diagnostic instead of
        // overflowing the stack. Non-partial includes keep the hard cycle error.
        if partial.is_some() {
            if self.partial_depth >= PARTIAL_RECURSION_LIMIT {
                return Err(diag_render(codes::PARTIAL_RECURSION).arg("limit", PARTIAL_RECURSION_LIMIT));
            }
        } else if self.include_stack.contains(&child.id) {
            let mut chain = self.include_stack.clone();
            chain.push(child.id);
            return Err(diag_load(codes::CIRCULAR_INCLUDE).arg("chain", chain.join(" -> ")));
        }

        let bound_scope = match partial {
            Some((params, _)) => Some(bind_partial(params, with)?),
            None => None,
        };

        // include 循环栈（canonical id 栈 — 能覆盖 A include B include A 的双向环）
        self.include_stack.push(child.id.clone());
        if partial.is_some() {
            self.partial_depth += 1;
        }
        // 保存当前模板的上下文，让被 include 模板的继承链独立解析
        let saved_id = std::mem::take(&mut self.current_id);
        let saved_blocks = std::mem::take(&mut self.block_levels);
        let saved_block_level = self.block_level;
        let saved_visiting_extends = std::mem::take(&mut self.visiting_extends);
        self.current_id = child.id.clone();

        // Include context: inherit by default, merge `with`, isolate on `only`.
        // A partial's scope is the bound parameter map only.
        let saved_scope = std::mem::take(&mut self.scope);
        match bound_scope {
            Some(scope) => self.scope = scope,
            None => {
                if !only {
                    self.scope = saved_scope.clone();
                }
                if let Some(Value::Object(entries)) = with {
                    for (key, value) in entries {
                        self.scope.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        // 走 render_doc：被 include 的模板也可能有 extends/block 链
        let out = match partial {
            Some((_, body)) => self.render_list(body),
            None => self.render_doc(&child.document),
        };

        // 恢复当前模板上下文
        self.include_stack.pop();
        if partial.is_some() {
            self.partial_depth -= 1;
        }
        self.current_id = saved_id;
        self.block_levels = saved_blocks;
        self.block_level = saved_block_level;
        self.visiting_extends = saved_visiting_extends;
        self.scope = saved_scope;
        out
    }

    fn render_nodes(&mut self, node: &IrNode) -> Result<String, IrError> {
        match node {
            IrNode::Template { children, .. } => self.render_list(children),
            IrNode::Text { value, .. } => Ok(value.clone()),
            IrNode::Comment { .. } => Ok(String::new()),
            IrNode::Interpolation { expression, raw, .. } => {
                let v = self.eval(expression)?;
                let s = value_to_string(&v);
                let skip_escape = *raw || self.is_raw_filter(expression) || self.is_macro_call(expression);
                if skip_escape { Ok(s) } else { Ok(html_escape(&s)) }
            }
            IrNode::StmtIf { test, consequent, else_ifs, alternate, .. } => {
                if is_truthy(&self.eval(test)?) {
                    return self.render_list(consequent);
                }
                for ei in else_ifs {
                    if let IrNode::StmtElseIf { test, consequent, .. } = ei {
                        if is_truthy(&self.eval(test)?) {
                            return self.render_list(consequent);
                        }
                    }
                }
                if let Some(alt) = alternate {
                    return self.render_list(alt);
                }
                Ok(String::new())
            }
            IrNode::StmtFor { item, index, iterable, body, .. } => {
                let iter_val = self.eval(iterable)?;
                let items = match iter_val {
                    Value::Array(a) => a,
                    other => return Err(diag_render(codes::INVALID_ITERABLE).arg("expected", "array").arg("got", format!("{other:?}"))),
                };
                let mut out = String::new();
                for (i, val) in items.into_iter().enumerate() {
                    let prev = self.scope.insert(item.clone(), val);
                    let prev_idx =
                        if let Some(idx) = index { self.scope.insert(idx.clone(), Value::Number(i.into())) } else { None };
                    out.push_str(&self.render_list(body)?);
                    restore(&mut self.scope, item, prev);
                    if let Some(idx) = index {
                        restore(&mut self.scope, idx, prev_idx);
                    }
                }
                Ok(out)
            }
            IrNode::StmtRaw { value, .. } => Ok(value.clone()),

            // partial：参数绑定发生在 include 时（render_loaded）；直接渲染时
            // 就是渲染其 body。
            IrNode::StmtPartial { body, .. } => self.render_list(body),

            // Macro declarations and static imports produce no output.
            IrNode::StmtMacro { .. } | IrNode::StmtImport { .. } => Ok(String::new()),

            // `call name(args)` … `end call`: invoke the macro with a caller
            // block captured in the current scope.
            IrNode::StmtCall { name, arguments, params, body, .. } => {
                let Some(mac) = self.lookup_macro(name) else {
                    return Err(diag_render(codes::UNKNOWN_FUNCTION).arg("name", name.clone()));
                };
                let args: Result<Vec<_>, _> = arguments.iter().map(|a| self.eval(a)).collect();
                let caller = CallerBlock {
                    body: body.clone(),
                    scope: self.scope.clone(),
                    module: self.current_id.clone(),
                    params: params.iter().map(|p| p.name.clone()).collect(),
                };
                self.call_macro(&mac, &args?, Some(caller)).map(|v| value_to_string(&v))
            }

            // block：最派生的层级定义优先，没有覆盖时渲染自身 body。
            IrNode::StmtBlock { name, body, .. } => {
                // 记录当前 block 名字，super 会用到
                let saved_block = self.current_block.take();
                self.current_block = Some(name.clone());
                let saved_level = self.block_level;

                let result = match self.block_levels.iter().position(|level| level.contains_key(name)) {
                    Some(idx) => {
                        self.block_level = idx;
                        let nodes = self.block_levels[idx][name].clone();
                        self.render_list(&nodes)
                    }
                    None => {
                        self.block_level = self.block_levels.len().saturating_sub(1);
                        self.render_list(body)
                    }
                };

                self.block_level = saved_level;
                self.current_block = saved_block;
                result
            }

            // super：渲染上一层继承级别中同名 block 的原始内容。
            IrNode::StmtSuper { .. } => {
                let block_name = self.current_block.clone().ok_or_else(|| diag_render(codes::SUPER_OUTSIDE_BLOCK))?;
                let target = self.block_level + 1;
                let parent_body = self
                    .block_levels
                    .get(target)
                    .and_then(|level| level.get(&block_name))
                    .cloned();
                match parent_body {
                    Some(body) => {
                        let saved_level = self.block_level;
                        self.block_level = target;
                        let rendered = self.render_list(&body);
                        self.block_level = saved_level;
                        rendered
                    }
                    // 根模板没有定义这个 block，输出空
                    None => Ok(String::new()),
                }
            }

            // include：加载并内联渲染。
            IrNode::StmtInclude { path, with, only, .. } => {
                let Some(path_str) = literal_path(path) else {
                    return Err(diag_load(codes::INCLUDE_PATH_NOT_LITERAL)
                        .template_id(self.current_id.clone()));
                };
                let explicit = match with {
                    Some(expr) => Some(self.eval(expr)?),
                    None => None,
                };
                self.render_loaded(path_str, "include", explicit.as_ref(), *only)
            }

            // extends：由 render_doc 集中处理，这里不应单独出现。
            IrNode::StmtExtends { .. } => {
                Err(diag_render(codes::EXTENDS_NOT_TOP_LEVEL))
            }

            other => Err(diag_render(codes::NOT_RENDERABLE_NODE).arg("node", format!("{other:?}"))),
        }
    }

    fn render_list(&mut self, nodes: &[IrNode]) -> Result<String, IrError> {
        // Whitespace control is resolved once per sibling list so every host
        // shares the same text before any node is rendered.
        let nodes = crate::trim::normalized_children(nodes);
        let mut out = String::new();
        for n in &nodes {
            out.push_str(&self.render_nodes(n)?);
        }
        Ok(out)
    }

    // ------------------------------------------------------------------
    // 表达式求值
    // ------------------------------------------------------------------

    fn eval(&mut self, expr: &IrNode) -> Result<Value, IrError> {
        match expr {
            IrNode::ExprLiteral { value, .. } => Ok(value.clone()),
            IrNode::ExprIdentifier { name, .. } => {
                match self.scope.get(name).cloned() {
                    Some(v) => Ok(v),
                    None if self.strict_undefined => {
                        Err(diag_render(codes::UNDEFINED_VARIABLE).arg("name", name.clone()))
                    }
                    None => Ok(Value::Null),
                }
            }
            IrNode::ExprMember { object, property, .. } => {
                let obj = self.eval(object)?;
                match obj {
                    Value::Object(map) => Ok(map.get(property).cloned().unwrap_or(Value::Null)),
                    _ => Ok(Value::Null),
                }
            }
            IrNode::ExprIndex { object, index, .. } => {
                let obj = self.eval(object)?;
                let idx = self.eval(index)?;
                match (obj, idx) {
                    (Value::Array(a), Value::Number(n)) => {
                        let i = n.as_u64().unwrap_or(0) as usize;
                        Ok(a.get(i).cloned().unwrap_or(Value::Null))
                    }
                    (Value::Object(map), Value::String(k)) => Ok(map.get(&k).cloned().unwrap_or(Value::Null)),
                    _ => Ok(Value::Null),
                }
            }
            IrNode::ExprCall { callee, arguments, .. } => {
                let IrNode::ExprIdentifier { name, .. } = callee.as_ref() else {
                    return Err(diag_render(codes::CALLS_NOT_SUPPORTED));
                };
                if name == "caller" {
                    let args: Result<Vec<_>, _> = arguments.iter().map(|a| self.eval(a)).collect();
                    return self.render_caller_block(&args?);
                }
                if let Some(mac) = self.lookup_macro(name) {
                    let args: Result<Vec<_>, _> = arguments.iter().map(|a| self.eval(a)).collect();
                    return self.call_macro(&mac, &args?, None);
                }
                let args: Result<Vec<_>, _> = arguments.iter().map(|a| self.eval(a)).collect();
                crate::builtins::call_function_with(name, &args?, self.capabilities, self.custom)
            }
            IrNode::ExprTest { expression, test, arguments, negated, .. } => {
                let value = self.eval(expression)?;
                let args: Result<Vec<_>, _> = arguments.iter().map(|a| self.eval(a)).collect();
                Ok(Value::Bool(crate::builtins::call_test_with(
                    test,
                    &value,
                    &args?,
                    *negated,
                    self.capabilities,
                    self.custom,
                )?))
            }
            IrNode::ExprBinary { operator, left, right, .. } => {
                let l = self.eval(left)?;
                let r = self.eval(right)?;
                eval_binary(operator, &l, &r)
            }
            IrNode::ExprUnary { operator, argument, .. } => {
                let v = self.eval(argument)?;
                match operator.as_str() {
                    "!" => Ok(Value::Bool(!is_truthy(&v))),
                    "-" => match v {
                        Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                Ok(Value::Number((-i).into()))
                            } else if let Some(f) = n.as_f64() {
                                Ok(serde_json::Number::from_f64(-f).map(Value::Number).unwrap_or(Value::Null))
                            } else {
                                Ok(Value::Null)
                            }
                        }
                        _ => Ok(Value::Null),
                    },
                    "+" => Ok(v),
                    _ => Err(diag_render(codes::UNKNOWN_UNARY).arg("operator", operator)),
                }
            }
            IrNode::ExprObject { entries, .. } => {
                let mut map = serde_json::Map::new();
                for entry in entries {
                    map.insert(entry.key.clone(), self.eval(&entry.value)?);
                }
                Ok(Value::Object(map))
            }
            IrNode::ExprPipe { expression, filter, arguments, .. } => {
                let mut val = self.eval(expression)?;
                let args: Result<Vec<_>, _> = arguments.iter().map(|a| self.eval(a)).collect();
                val = apply_filter(filter, val, &args?, self.capabilities, self.custom)?;
                Ok(val)
            }
            _ => Err(diag_render(codes::INVALID_EXPRESSION_NODE)),
        }
    }
}

// ---------------------------------------------------------------------------
// 独立辅助函数
// ---------------------------------------------------------------------------

fn json_object_to_map(ctx: &Value) -> Result<HashMap<String, Value>, IrError> {
    match ctx {
        Value::Object(map) => Ok(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
        _ => Err(diag_render(codes::CONTEXT_NOT_OBJECT)),
    }
}

/// Human-readable JSON kind for diagnostics.
fn json_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// `include` / `extends` targets must be string literals: the loader only
/// resolves build-time catalog entries, never any dynamic path.
fn literal_path(node: &IrNode) -> Option<&str> {
    match node {
        IrNode::ExprLiteral { value: Value::String(s), .. } => Some(s),
        _ => None,
    }
}

fn restore(scope: &mut HashMap<String, Value>, key: &str, prev: Option<Value>) {
    match prev {
        Some(v) => {
            scope.insert(key.to_string(), v);
        }
        None => {
            scope.remove(key);
        }
    }
}

/// If `body` is a parameterized partial template, return its declaration.
fn partial_parts(body: &IrNode) -> Option<(&[crate::types::PartialParam], &[IrNode])> {
    if let IrNode::Template { children, .. } = body {
        for c in children {
            if let IrNode::StmtPartial { params, body, .. } = c {
                return Some((params, body));
            }
        }
    }
    None
}

/// Bind a partial's declared parameters from the explicit `with` object.
///
/// The caller context is never consulted: a parameter is either present in
/// `with`, has a literal default, or is a hard error. Undeclared keys are
/// rejected so typos surface at render time.
fn bind_partial(
    params: &[crate::types::PartialParam],
    with: Option<&Value>,
) -> Result<HashMap<String, Value>, IrError> {
    let entries = match with {
        Some(Value::Object(map)) => Some(map),
        _ => None,
    };
    let mut scope = HashMap::new();
    for param in params {
        match entries.and_then(|m| m.get(&param.name)) {
            Some(value) => {
                scope.insert(param.name.clone(), value.clone());
            }
            None => match &param.default {
                Some(IrNode::ExprLiteral { value, .. }) => {
                    scope.insert(param.name.clone(), value.clone());
                }
                Some(_) => return Err(diag_render(codes::INVALID_EXPRESSION_NODE)),
                None => {
                    return Err(diag_render(codes::MISSING_PARAMETER).arg("name", param.name.clone()));
                }
            },
        }
    }
    if let Some(map) = entries {
        for key in map.keys() {
            if !params.iter().any(|p| &p.name == key) {
                return Err(diag_render(codes::UNKNOWN_PARAMETER).arg("name", key.clone()));
            }
        }
    }
    Ok(scope)
}

fn eval_binary(op: &str, l: &Value, r: &Value) -> Result<Value, IrError> {
    match op {
        "+" => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(num_op(a, b, |x, y| x + y)),
            _ => Ok(Value::String(format!("{}{}", value_to_string(l), value_to_string(r)))),
        },
        "-" => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(num_op(a, b, |x, y| x - y)),
            _ => Ok(Value::Null),
        },
        "*" => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(num_op(a, b, |x, y| x * y)),
            _ => Ok(Value::Null),
        },
        "/" => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(num_op(a, b, |x, y| x / y)),
            _ => Ok(Value::Null),
        },
        "%" => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(num_op(a, b, |x, y| x % y)),
            _ => Ok(Value::Null),
        },
        "==" => Ok(Value::Bool(l == r)),
        "!=" => Ok(Value::Bool(l != r)),
        "<" | "<=" | ">" | ">=" => match (as_f64(l), as_f64(r)) {
            (Some(a), Some(b)) => Ok(Value::Bool(match op {
                "<" => a < b,
                "<=" => a <= b,
                ">" => a > b,
                ">=" => a >= b,
                _ => false,
            })),
            _ => Ok(Value::Bool(false)),
        },
        "&&" => Ok(Value::Bool(is_truthy(l) && is_truthy(r))),
        "||" => Ok(Value::Bool(is_truthy(l) || is_truthy(r))),
        "in" => match r {
            Value::Array(a) => Ok(Value::Bool(a.contains(l))),
            Value::String(s) => Ok(Value::Bool(s.contains(&value_to_string(l)))),
            Value::Object(map) => Ok(Value::Bool(map.contains_key(&value_to_string(l)))),
            _ => Ok(Value::Bool(false)),
        },
        _ => Err(diag_render(codes::UNKNOWN_BINARY).arg("operator", op)),
    }
}

fn num_op(a: &serde_json::Number, b: &serde_json::Number, f: impl Fn(f64, f64) -> f64) -> Value {
    match (as_f64(&Value::Number(a.clone())), as_f64(&Value::Number(b.clone()))) {
        (Some(x), Some(y)) => serde_json::Number::from_f64(f(x, y)).map(Value::Number).unwrap_or(Value::Null),
        _ => Value::Null,
    }
}

fn as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        _ => None,
    }
}

fn apply_filter(
    name: &str,
    value: Value,
    args: &[Value],
    capabilities: CapabilitySet,
    custom: Option<&dyn CustomRegistry>,
) -> Result<Value, IrError> {
    let spec = match validate_filter(name, args.len(), capabilities) {
        Ok(spec) => spec,
        Err(FilterLookupError::Unknown) => {
            let Some(registry) = custom else {
                return Err(diag_render(codes::UNKNOWN_FILTER).arg("filter", name));
            };
            let Some(entry) = custom_entry(registry.entries(), Namespace::Filter, name) else {
                return Err(diag_render(codes::UNKNOWN_FILTER).arg("filter", name));
            };
            if args.len() < entry.min_args || args.len() > entry.max_args {
                return Err(diag_render(codes::FILTER_ARITY)
                    .arg("filter", entry.name.clone())
                    .arg("minArgs", entry.min_args as i64)
                    .arg("maxArgs", entry.max_args as i64)
                    .arg("got", args.len() as i64));
            }
            if !capabilities.allows(entry.capability) {
                return Err(capability_denied(&entry.name, entry.capability));
            }
            return registry
                .apply_filter(&entry.name, &value, args)
                .map_err(|error| custom_failure(Namespace::Filter, &entry.name, error));
        }
        Err(FilterLookupError::Arity { spec }) => {
            return Err(diag_render(codes::FILTER_ARITY)
                .arg("filter", spec.name)
                .arg("minArgs", spec.min_args as i64)
                .arg("maxArgs", spec.max_args as i64)
                .arg("got", args.len() as i64));
        }
        Err(FilterLookupError::NotAllowed { spec }) => {
            return Err(capability_denied(spec.name, spec.capability));
        }
    };
    apply_builtin_filter(spec.name, value, args)
}

fn apply_builtin_filter(name: &str, value: Value, args: &[Value]) -> Result<Value, IrError> {
    match name {
        // ——大小写——
        "uppercase" => Ok(Value::String(value_to_string(&value).to_uppercase())),
        "lowercase" => Ok(Value::String(value_to_string(&value).to_lowercase())),
        "trim" => Ok(Value::String(value_to_string(&value).trim().to_string())),
        "default" => {
            if matches!(value, Value::Null) || matches!(&value, Value::String(s) if s.is_empty()) {
                Ok(args.first().cloned().unwrap_or(Value::Null))
            } else {
                Ok(value)
            }
        }
        "length" => match value {
            Value::String(s) => Ok(Value::Number(s.chars().count().into())),
            Value::Array(a) => Ok(Value::Number(a.len().into())),
            Value::Object(o) => Ok(Value::Number(o.len().into())),
            _ => Ok(Value::Number(0.into())),
        },
        "join" => {
            let sep = args.first().map(value_to_string).unwrap_or_else(|| ",".into());
            match value {
                Value::Array(a) => Ok(Value::String(a.iter().map(value_to_string).collect::<Vec<_>>().join(&sep))),
                _ => Ok(Value::String(value_to_string(&value))),
            }
        }
        "replace" => {
            let from = args.first().map(value_to_string).unwrap_or_default();
            let to = args.get(1).map(value_to_string).unwrap_or_default();
            Ok(Value::String(value_to_string(&value).replace(&from, &to)))
        }
        // ——HTML escape 过滤器：内部做 escape，renderer 识别为 safe 不再二次 escape——
        "escape" => Ok(Value::String(html_escape(&value_to_string(&value)))),
        "safe" => Ok(value),
        // ——日期（最小 strftime：%Y %m %d %H %M %S %%）——
        "date" => {
            let pattern = args.first().map(value_to_string).unwrap_or_else(|| "%Y-%m-%d".into());
            let input_str = value_to_string(&value);
            Ok(Value::String(format_date(&input_str, &pattern)))
        }
        // ——slug：ASCII-fold + 去重分隔——
        "slug" => Ok(Value::String(slugify(&value_to_string(&value)))),
        other => Err(diag_render(codes::UNKNOWN_FILTER).arg("filter", other)),
    }
}

/// 判断管道结果是否应跳过自动 escape。

/// `dejavu.template.capability-not-allowed` for an entry the host did not grant.
pub(crate) fn capability_denied(name: &str, capability: Capability) -> IrError {
    diag_render(codes::CAPABILITY_NOT_ALLOWED)
        .arg("name", name)
        .arg("capability", capability.as_str())
}

/// `dejavu.template.custom-invocation` for a host implementation that failed.
pub(crate) fn custom_failure(namespace: Namespace, name: &str, error: CustomError) -> IrError {
    diag_render(codes::CUSTOM_INVOCATION)
        .arg("namespace", namespace.as_str())
        .arg("name", name)
        .arg("message", error.message)
}

/// 最小 strftime — 支持 `%Y %m %d %H %M %S %%`。
///
/// 输入为 ISO 8601 字符串、epoch 数字（毫秒）或字面量 `"now"`。
/// 无法解析时返回空字符串（与 TS `formatDate` 行为一致）。
fn format_date(input: &str, pattern: &str) -> String {
    let dt = match input {
        "" => return String::new(),
        "now" => chrono_now(),
        s if s.chars().all(|c| c.is_ascii_digit()) => chrono_from_epoch_ms(s),
        s => chrono_from_iso(s),
    };
    let Some((y, mo, d, h, mi, se)) = dt else {
        return String::new();
    };
    let pad = |n: u32| format!("{:02}", n);
    pattern
        .replace("%%", "\x00")
        .replace("%Y", &y.to_string())
        .replace("%m", &pad(mo))
        .replace("%d", &pad(d))
        .replace("%H", &pad(h))
        .replace("%M", &pad(mi))
        .replace("%S", &pad(se))
        .replace('\x00', "%")
}

fn chrono_now() -> Option<(i32, u32, u32, u32, u32, u32)> {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    chrono_from_epoch(d.as_secs() as i64)
}

fn chrono_from_epoch_ms(s: &str) -> Option<(i32, u32, u32, u32, u32, u32)> {
    let ms: i64 = s.parse().ok()?;
    chrono_from_epoch(ms / 1000)
}

fn chrono_from_epoch(secs: i64) -> Option<(i32, u32, u32, u32, u32, u32)> {
    // 简单 UTC epoch 分解（无 timezone）。
    // 闰年规则：4年一闰，百年不闰，四百又闰。
    let mut t = secs.checked_sub(0)?; // clamp negative
    let sec = (t % 60) as u32;
    t /= 60;
    let min = (t % 60) as u32;
    t /= 60;
    let hour = (t % 24) as u32;
    t /= 24;
    let days_since_epoch = t;

    let mut year: i32 = 1970;
    let mut d = days_since_epoch;
    loop {
        let leap = is_leap_year(year);
        let ydays = if leap { 366 } else { 365 };
        if d < ydays { break; }
        d -= ydays;
        year += 1;
        if year > 9999 { return None; }
    }
    let (month, mday) = month_and_day(year, d as u32)?;
    Some((year, month, mday, hour, min, sec))
}

fn is_leap_year(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn month_and_day(year: i32, mut day_in_year: u32) -> Option<(u32, u32)> {
    let days_per = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let leap = is_leap_year(year);
    for (i, &dm) in days_per.iter().enumerate() {
        let dm = dm + (if leap && i == 1 { 1 } else { 0 });
        if day_in_year < dm {
            return Some(((i + 1) as u32, (day_in_year + 1)));
        }
        day_in_year -= dm;
    }
    None
}

fn chrono_from_iso(s: &str) -> Option<(i32, u32, u32, u32, u32, u32)> {
    // 极简 ISO 8601: 支持 2024-01-15 / 2024-01-15T12:30:45 / 2024-01-15T12:30:45Z / 2024-01-15T12:30:45+08:00
    let s = s.trim();
    let parts: Vec<&str> = s.split(['T', ' ']).collect();
    let date = parts.first()?;
    let (y_str, mo_str, d_str) = date.split_once('-')
        .and_then(|(a, b)| {
            let (mo, d) = b.split_once('-')?;
            Some((a, mo, d))
        })?;
    let year: i32 = y_str.parse().ok()?;
    let month: u32 = mo_str.parse().ok()?;
    let day: u32 = d_str.parse().ok()?;

    // 验证月/日范围 —— TS `new Date("2024-99-99")` 是 Invalid Date，format 后为空串
    if !(1..=12).contains(&month) {
        return None;
    }
    if !(1..=days_in_month(year, month)).contains(&day) {
        return None;
    }

    let mut hour: u32 = 0;
    let mut min: u32 = 0;
    let mut sec: u32 = 0;
    if let Some(t) = parts.get(1) {
        let t_clean = t.trim_end_matches('Z').split('+').next()?.split('-').next()?;
        let tp: Vec<&str> = t_clean.split(':').collect();
        hour = tp.first().and_then(|v| v.parse().ok()).unwrap_or(0);
        min = tp.get(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        // `...:05.000Z` 的秒字段可能带小数部分，只取整数秒。
        sec = tp
            .get(2)
            .and_then(|v| v.split(['.', ',']).next())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
    }

    // 时间字段也要合法
    if hour >= 24 || min >= 60 || sec >= 60 {
        return None;
    }

    Some((year, month, day, hour, min, sec))
}

/// 返回某年某月的天数（考虑闰年）。
fn days_in_month(year: i32, month: u32) -> u32 {
    let days_per: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let d = days_per[(month - 1) as usize];
    if month == 2 && is_leap_year(year) { d + 1 } else { d }
}

/// ASCII-fold slugify（匹配 TS `slugify`）。
///
/// 1. NFKD（compatibility + canonical decomposition）规范化后去除组合字符；
/// 2. 小写化；
/// 3. 非 `[a-z0-9]` 序列压缩为单个 `-`；
/// 4. 裁掉首尾 `-`。
///
/// 使用 NFKD 而非 NFD：NFKD 把连字（`ﬃ` → `ffi`）和全角字符（`Ａ` → `A`）做兼容分解，
/// TS `normalize("NFKD")` 也是这个语义。
fn slugify(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    // 手写 combining mark 过滤（unicode-normalization 0.1 暴露 nfkd()，
    // 但 is_combining_mark 在不同 rustc 上 API 位置不同）。
    const COMBINING_START: u32 = 0x0300;
    const COMBINING_END: u32 = 0x036F;
    const COMBINING_EXTENDED_START: u32 = 0x1AB0;
    const COMBINING_EXTENDED_END: u32 = 0x1AFF;
    const COMBINING_SUPP_START: u32 = 0x1DC0;
    const COMBINING_SUPP_END: u32 = 0x1DFF;
    const COMBINING_MARK_START: u32 = 0x20D0;
    const COMBINING_MARK_END: u32 = 0x20FF;
    const COMBINING_SYMBOLS_START: u32 = 0xFE20;
    const COMBINING_SYMBOLS_END: u32 = 0xFE2F;

    let is_combining = |c: char| -> bool {
        let cp = c as u32;
        matches!(cp, COMBINING_START..=COMBINING_END)
            || matches!(cp, COMBINING_EXTENDED_START..=COMBINING_EXTENDED_END)
            || matches!(cp, COMBINING_SUPP_START..=COMBINING_SUPP_END)
            || matches!(cp, COMBINING_MARK_START..=COMBINING_MARK_END)
            || matches!(cp, COMBINING_SYMBOLS_START..=COMBINING_SYMBOLS_END)
    };

    let folded: String = s
        .nfkd()
        .filter(|c| !is_combining(*c))
        .collect::<String>()
        .to_lowercase();
    let mut out = String::new();
    let mut prev_dash = false;
    for c in folded.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => {
            // 与 JS `String(number)` 对齐：整数值不输出 `.0`。
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(u) = n.as_u64() {
                u.to_string()
            } else if let Some(f) = n.as_f64() {
                if f.is_finite() && f.fract() == 0.0 && f.abs() < 1e21 {
                    format!("{f:.0}")
                } else {
                    n.to_string()
                }
            } else {
                n.to_string()
            }
        }
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn html_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            c => c.to_string(),
        })
        .collect()
}
