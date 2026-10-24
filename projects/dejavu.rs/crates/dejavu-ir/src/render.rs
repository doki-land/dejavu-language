use crate::{IrError, codes, diag_load, diag_render};
use crate::types::{IrDocument, IrNode};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// RenderOptions：宿主可选渲染参数（严格模式、依赖追踪等）
// ---------------------------------------------------------------------------

/// 渲染选项 —— 匹配 TS `RenderOptions`。
///
/// 默认 `strict_undefined = false`（缺失变量 → null → 空串）。
#[derive(Debug, Clone, Default)]
pub struct RenderOptions {
    /// 开启后，引用未定义变量抛 `UNDEFINED_VARIABLE` 错误。
    pub strict_undefined: bool,
    /// 模板 canonical id（用于 include 循环检测和依赖追踪）。
    pub name: Option<String>,
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
pub fn render_ir_opts(
    doc: &IrDocument,
    ctx: &Value,
    opts: RenderOptions,
    loader: Option<&mut dyn TemplateLoader>,
) -> Result<String, IrError> {
    let scope = json_object_to_map(ctx)?;
    let default_id = opts.name.clone().unwrap_or_else(|| "<main>".into());
    let mut r = Renderer {
        scope,
        loader,
        strict_undefined: opts.strict_undefined,
        current_id: default_id,
        blocks: HashMap::new(),
        parent_blocks: HashMap::new(),
        current_block: None,
        visiting_extends: HashSet::new(),
        include_stack: Vec::new(),
    };
    r.render_doc(doc)
}

// ---------------------------------------------------------------------------
// 内部 Renderer
// ---------------------------------------------------------------------------

struct Renderer<'a> {
    scope: HashMap<String, Value>,
    loader: Option<&'a mut dyn TemplateLoader>,
    strict_undefined: bool,
    current_id: String,
    blocks: HashMap<String, Vec<IrNode>>,
    parent_blocks: HashMap<String, Vec<IrNode>>,
    current_block: Option<String>,
    /// extends 访问过的 canonical id（防止继承环）
    visiting_extends: HashSet<String>,
    /// include 递归栈（防止 include 环）
    include_stack: Vec<String>,
}

impl<'a> Renderer<'a> {
    fn render_doc(&mut self, doc: &IrDocument) -> Result<String, IrError> {
        // 特殊处理 Template body：先收集 block 定义，处理 extends，再渲染。
        if let IrNode::Template { children, .. } = &doc.body {
            let mut extends_path: Option<String> = None;
            let mut child_blocks: Vec<(String, Vec<IrNode>)> = Vec::new();

            for c in children {
                match c {
                    IrNode::StmtExtends { parent, .. } => {
                        extends_path = Some(value_to_string(&self.eval(parent)?));
                    }
                    IrNode::StmtBlock { name, body, .. } => {
                        child_blocks.push((name.clone(), body.clone()));
                    }
                    _ => {}
                }
            }

            if let Some(parent_str) = extends_path {
                let loader = self
                    .loader
                    .as_deref_mut()
                    .ok_or_else(|| diag_load(codes::MISSING_LOADER).arg("feature", "extends"))?;
                let parent = loader.resolve(&parent_str, Some(&self.current_id))?;

                // canonical id 层面的循环检测
                if self.visiting_extends.contains(&parent.id) {
                    return Err(diag_load(codes::CIRCULAR_EXTENDS).arg("chain", parent.id.clone()));
                }
                self.visiting_extends.insert(parent.id.clone());

                // 提取 parent 原始 blocks，供 super 使用
                let parent_body_blocks = extract_blocks(&parent.document.body);

                // 保存会被覆盖的状态（self.blocks 不再 take —— 整条 extends 链累积合并）
                let saved_parent_blocks = std::mem::take(&mut self.parent_blocks);
                let saved_id = std::mem::take(&mut self.current_id);

                // child blocks 合并到 self.blocks：底层优先，上层 block 不覆盖下层已定义的 key
                for (name, body) in &child_blocks {
                    self.blocks.entry(name.clone()).or_insert_with(|| body.clone());
                }
                self.parent_blocks = parent_body_blocks;
                self.current_id = parent.id.clone();

                let out = self.render_doc(&parent.document)?;

                // 恢复（self.blocks 保留累积结果 —— 整条链的最具体 override）
                self.visiting_extends.remove(&parent.id);
                self.parent_blocks = saved_parent_blocks;
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

        let loader = self
            .loader
            .as_deref_mut()
            .ok_or_else(|| diag_load(codes::MISSING_LOADER).arg("feature", feature))?;
        let child = loader.resolve(path, Some(&self.current_id))?;

        // include 循环检测（canonical id 栈 — 能覆盖 A include B include A 的双向环）
        if self.include_stack.contains(&child.id) {
            let mut chain = self.include_stack.clone();
            chain.push(child.id);
            return Err(diag_load(codes::CIRCULAR_INCLUDE)
                .arg("chain", chain.join(" -> ")));
        }
        self.include_stack.push(child.id.clone());
        // 保存当前模板的上下文，让被 include 模板的继承链独立解析
        let saved_id = std::mem::take(&mut self.current_id);
        let saved_blocks = std::mem::take(&mut self.blocks);
        let saved_parent_blocks = std::mem::take(&mut self.parent_blocks);
        let saved_visiting_extends = std::mem::take(&mut self.visiting_extends);
        self.current_id = child.id.clone();

        // Include context: inherit by default, merge `with`, isolate on `only`.
        let saved_scope = std::mem::take(&mut self.scope);
        if !only {
            self.scope = saved_scope.clone();
        }
        if let Some(Value::Object(entries)) = with {
            for (key, value) in entries {
                self.scope.insert(key.clone(), value.clone());
            }
        }

        // 走 render_doc：被 include 的模板也可能有 extends/block 链
        let out = self.render_doc(&child.document);

        // 恢复当前模板上下文
        self.include_stack.pop();
        self.current_id = saved_id;
        self.blocks = saved_blocks;
        self.parent_blocks = saved_parent_blocks;
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
                let skip_escape = *raw || is_raw_filter(expression);
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

            // block：如果有覆盖内容就渲染覆盖，否则渲染自身 body。
            IrNode::StmtBlock { name, body, .. } => {
                // 记录当前 block 名字，super 会用到
                let saved = self.current_block.take();
                self.current_block = Some(name.clone());

                let result = match self.blocks.get(name).cloned() {
                    Some(override_nodes) => self.render_list(&override_nodes),
                    None => self.render_list(body),
                };

                self.current_block = saved;
                result
            }

            // super：渲染 parent 同名 block 的原始内容。
            IrNode::StmtSuper { .. } => {
                let block_name = self.current_block.clone().ok_or_else(|| diag_render(codes::SUPER_OUTSIDE_BLOCK))?;
                match self.parent_blocks.get(&block_name).cloned() {
                    Some(parent_body) => self.render_list(&parent_body),
                    None => Ok(String::new()), // parent 没有定义这个 block，输出空
                }
            }

            // include：加载并内联渲染。
            IrNode::StmtInclude { path, with, only, .. } => {
                let p = self.eval(path)?;
                let path_str = value_to_string(&p);
                let explicit = match with {
                    Some(expr) => Some(self.eval(expr)?),
                    None => None,
                };
                self.render_loaded(&path_str, "include", explicit.as_ref(), *only)
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

    fn eval(&self, expr: &IrNode) -> Result<Value, IrError> {
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
            IrNode::ExprCall { .. } => Err(diag_render(codes::CALLS_NOT_SUPPORTED)),
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
            IrNode::ExprPipe { expression, filter, arguments, .. } => {
                let mut val = self.eval(expression)?;
                let args: Result<Vec<_>, _> = arguments.iter().map(|a| self.eval(a)).collect();
                val = apply_filter(filter, val, &args?)?;
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

/// 从文档 body 中提取所有顶层 `block` 定义。
fn extract_blocks(body: &IrNode) -> HashMap<String, Vec<IrNode>> {
    let mut out = HashMap::new();
    if let IrNode::Template { children, .. } = body {
        for c in children {
            if let IrNode::StmtBlock { name, body, .. } = c {
                out.insert(name.clone(), body.clone());
            }
        }
    }
    out
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

fn apply_filter(name: &str, value: Value, args: &[Value]) -> Result<Value, IrError> {
    match name {
        // ——大小写：全名 + 别名——
        "uppercase" | "upper" => Ok(Value::String(value_to_string(&value).to_uppercase())),
        "lowercase" | "lower" => Ok(Value::String(value_to_string(&value).to_lowercase())),
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
        "escape" | "e" => Ok(Value::String(html_escape(&value_to_string(&value)))),
        "raw" | "safe" => Ok(value),
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
///
/// `escape`/`e` 在 filter 内部已做 html_escape，不能再二次 escape；
/// `raw`/`safe` 是显式跳过 escape 的意图。
fn is_raw_filter(expr: &IrNode) -> bool {
    matches!(expr, IrNode::ExprPipe { filter, .. }
        if matches!(filter.as_str(), "raw" | "safe" | "escape" | "e"))
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
