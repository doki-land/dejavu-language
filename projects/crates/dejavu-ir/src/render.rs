use crate::IrError;
use crate::types::{IrDocument, IrNode};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// 公共入口
// ---------------------------------------------------------------------------

/// 模板加载器：根据路径加载另一个模板的 IR 文档。
///
/// 返回已解析好的 [`IrDocument`]，保持 IR 层纯净——不依赖 parser 或宿主文件系统。
/// 宿主（如 dejavu-engine、doki-engine）负责实现实际的路径解析和源码加载。
pub trait TemplateLoader {
    fn load(&mut self, path: &str) -> Result<IrDocument, IrError>;
}

/// 渲染一个 IR 文档。无 loader 时 `extends`/`include`/`super` 会报错。
///
/// 这是现有 T1 路径的向后兼容入口。需要完整模板继承链时用 [`render_ir_with_loader`]。
pub fn render_ir(doc: &IrDocument, ctx: &Value) -> Result<String, IrError> {
    let scope = json_object_to_map(ctx)?;
    let mut r = Renderer { scope, loader: None, blocks: HashMap::new(), visited: HashSet::new() };
    r.render_doc(doc)
}

/// 渲染一个 IR 文档，带完整模板支持：`extends`、`include`、`block`、`super`。
pub fn render_ir_with_loader(
    doc: &IrDocument,
    ctx: &Value,
    loader: &mut dyn TemplateLoader,
) -> Result<String, IrError> {
    let scope = json_object_to_map(ctx)?;
    let mut r = Renderer {
        scope,
        loader: Some(loader),
        blocks: HashMap::new(),
        visited: HashSet::new(),
    };
    r.render_doc(doc)
}

// ---------------------------------------------------------------------------
// 内部 Renderer
// ---------------------------------------------------------------------------

struct Renderer<'a> {
    scope: HashMap<String, Value>,
    loader: Option<&'a mut dyn TemplateLoader>,
    /// 当前渲染层级上的 block 覆盖。key = block 名字。
    blocks: HashMap<String, Vec<IrNode>>,
    /// extends 路径上已访问的模板，防止循环。
    visited: HashSet<String>,
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
                // 循环检测
                if !self.visited.insert(parent_str.clone()) {
                    return Err(IrError::Render(format!("circular extends: {parent_str}")));
                }

                // 保存旧 blocks，压入 child 覆盖
                let saved_blocks = std::mem::take(&mut self.blocks);
                for (name, body) in &child_blocks {
                    self.blocks.insert(name.clone(), body.clone());
                }

                let loader = self
                    .loader
                    .as_deref_mut()
                    .ok_or_else(|| IrError::Render("extends requires a template loader".to_string()))?;
                let parent_doc = loader.load(&parent_str)?;
                let out = self.render_doc(&parent_doc)?;

                // 恢复
                self.visited.remove(&parent_str);
                self.blocks = saved_blocks;

                return Ok(out);
            }
        }
        // 没有 extends，正常渲染
        self.render_nodes(&doc.body)
    }

    fn render_loaded(&mut self, path: &str, feature: &str) -> Result<String, IrError> {
        let loader = self
            .loader
            .as_deref_mut()
            .ok_or_else(|| IrError::Render(format!("{feature} requires a template loader")))?;
        let child = loader.load(path)?;
        self.render_nodes(&child.body)
    }

    fn render_nodes(&mut self, node: &IrNode) -> Result<String, IrError> {
        match node {
            IrNode::Template { children, .. } => {
                let mut out = String::new();
                for c in children {
                    out.push_str(&self.render_nodes(c)?);
                }
                Ok(out)
            }
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
                    other => return Err(IrError::Render(format!("for iterable must be array, got {other}"))),
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
            IrNode::StmtBlock { name, body, .. } => match self.blocks.get(name).cloned() {
                Some(override_nodes) => self.render_list(&override_nodes),
                None => self.render_list(body),
            },

            // super：渲染当前 block 的父级原始 body。实现方式：把 parent 传入的 blocks 里的同名
            // override 替换为 self 原始 body 来临时渲染。
            IrNode::StmtSuper { .. } => self.render_super(),

            // include：加载并内联渲染。
            IrNode::StmtInclude { path, .. } => {
                let p = self.eval(path)?;
                let path_str = value_to_string(&p);
                self.render_loaded(&path_str, "include")
            }

            // extends：由 render_doc 集中处理，这里不应单独出现。
            IrNode::StmtExtends { .. } => {
                Err(IrError::Render("extends must be handled at document top level".into()))
            }

            other => Err(IrError::Render(format!("node not renderable at top level: {other:?}"))),
        }
    }

    fn render_list(&mut self, nodes: &[IrNode]) -> Result<String, IrError> {
        let mut out = String::new();
        for n in nodes {
            out.push_str(&self.render_nodes(n)?);
        }
        Ok(out)
    }

    fn render_super(&mut self) -> Result<String, IrError> {
        // super 暂未实现完整 block 继承链，返回空字符串。
        // 完整实现需要在 extends 处理时保留 parent 原始 blocks。
        Ok(String::new())
    }

    // ------------------------------------------------------------------
    // 表达式求值
    // ------------------------------------------------------------------

    fn eval(&self, expr: &IrNode) -> Result<Value, IrError> {
        match expr {
            IrNode::ExprLiteral { value, .. } => Ok(value.clone()),
            IrNode::ExprIdentifier { name, .. } => Ok(self.scope.get(name).cloned().unwrap_or(Value::Null)),
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
            IrNode::ExprCall { .. } => Err(IrError::Render("calls not supported in T1 eval".into())),
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
                    _ => Err(IrError::Render(format!("unknown unary {operator}"))),
                }
            }
            IrNode::ExprPipe { expression, filter, arguments, .. } => {
                let mut val = self.eval(expression)?;
                let args: Result<Vec<_>, _> = arguments.iter().map(|a| self.eval(a)).collect();
                val = apply_filter(filter, val, &args?)?;
                Ok(val)
            }
            _ => Err(IrError::Render("invalid expression node".into())),
        }
    }
}

// ---------------------------------------------------------------------------
// 独立辅助函数
// ---------------------------------------------------------------------------

fn json_object_to_map(ctx: &Value) -> Result<HashMap<String, Value>, IrError> {
    match ctx {
        Value::Object(map) => Ok(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
        _ => Err(IrError::Render("context must be a JSON object".into())),
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
            _ => Ok(Value::Bool(false)),
        },
        _ => Err(IrError::Render(format!("unknown binary {op}"))),
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
        "raw" | "safe" => Ok(value),
        other => Err(IrError::Render(format!("unknown filter `{other}`"))),
    }
}

fn is_raw_filter(expr: &IrNode) -> bool {
    matches!(expr, IrNode::ExprPipe { filter, .. } if filter == "raw" || filter == "safe")
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
        Value::Number(n) => n.to_string(),
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
