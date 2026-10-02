use crate::IrError;
use crate::types::{IrDocument, IrNode};
use serde_json::Value;
use std::collections::HashMap;

/// Render an IR document with a JSON object context.
pub fn render_ir(doc: &IrDocument, ctx: &Value) -> Result<String, IrError> {
    let mut scope = json_object_to_map(ctx)?;
    render_nodes(&doc.body, &mut scope)
}

fn json_object_to_map(ctx: &Value) -> Result<HashMap<String, Value>, IrError> {
    match ctx {
        Value::Object(map) => Ok(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
        _ => Err(IrError::Render("context must be a JSON object".into())),
    }
}

fn render_nodes(node: &IrNode, scope: &mut HashMap<String, Value>) -> Result<String, IrError> {
    match node {
        IrNode::Template { children, .. } => {
            let mut out = String::new();
            for c in children {
                out.push_str(&render_nodes(c, scope)?);
            }
            Ok(out)
        }
        IrNode::Text { value, .. } => Ok(value.clone()),
        IrNode::Comment { .. } => Ok(String::new()),
        IrNode::Interpolation { expression, raw, .. } => {
            let v = eval(expression, scope)?;
            let s = value_to_string(&v);
            if *raw { Ok(s) } else { Ok(html_escape(&s)) }
        }
        IrNode::StmtIf { test, consequent, else_ifs, alternate, .. } => {
            if is_truthy(&eval(test, scope)?) {
                return render_list(consequent, scope);
            }
            for ei in else_ifs {
                if let IrNode::StmtElseIf { test, consequent, .. } = ei {
                    if is_truthy(&eval(test, scope)?) {
                        return render_list(consequent, scope);
                    }
                }
            }
            if let Some(alt) = alternate {
                return render_list(alt, scope);
            }
            Ok(String::new())
        }
        IrNode::StmtFor { item, index, iterable, body, .. } => {
            let iter_val = eval(iterable, scope)?;
            let items = match iter_val {
                Value::Array(a) => a,
                other => return Err(IrError::Render(format!("for iterable must be array, got {other}"))),
            };
            let mut out = String::new();
            for (i, val) in items.into_iter().enumerate() {
                let prev = scope.insert(item.clone(), val);
                let prev_idx = if let Some(idx) = index { scope.insert(idx.clone(), Value::Number(i.into())) } else { None };
                out.push_str(&render_list(body, scope)?);
                restore(scope, item, prev);
                if let Some(idx) = index {
                    restore(scope, idx, prev_idx);
                }
            }
            Ok(out)
        }
        IrNode::StmtRaw { value, .. } => Ok(value.clone()),
        IrNode::StmtBlock { body, .. } => render_list(body, scope),
        IrNode::StmtExtends { .. } | IrNode::StmtInclude { .. } | IrNode::StmtSuper { .. } => {
            Err(IrError::Render("extends/include/super require a template loader (not in T1 minimal render)".into()))
        }
        other => Err(IrError::Render(format!("node not renderable at top level: {other:?}"))),
    }
}

fn render_list(nodes: &[IrNode], scope: &mut HashMap<String, Value>) -> Result<String, IrError> {
    let mut out = String::new();
    for n in nodes {
        out.push_str(&render_nodes(n, scope)?);
    }
    Ok(out)
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

fn eval(expr: &IrNode, scope: &HashMap<String, Value>) -> Result<Value, IrError> {
    match expr {
        IrNode::ExprLiteral { value, .. } => Ok(value.clone()),
        IrNode::ExprIdentifier { name, .. } => Ok(scope.get(name).cloned().unwrap_or(Value::Null)),
        IrNode::ExprMember { object, property, .. } => {
            let obj = eval(object, scope)?;
            match obj {
                Value::Object(map) => Ok(map.get(property).cloned().unwrap_or(Value::Null)),
                _ => Ok(Value::Null),
            }
        }
        IrNode::ExprIndex { object, index, .. } => {
            let obj = eval(object, scope)?;
            let idx = eval(index, scope)?;
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
            let l = eval(left, scope)?;
            let r = eval(right, scope)?;
            eval_binary(operator, &l, &r)
        }
        IrNode::ExprUnary { operator, argument, .. } => {
            let v = eval(argument, scope)?;
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
            let mut val = eval(expression, scope)?;
            let args: Result<Vec<_>, _> = arguments.iter().map(|a| eval(a, scope)).collect();
            val = apply_filter(filter, val, &args?)?;
            Ok(val)
        }
        _ => Err(IrError::Render("invalid expression node".into())),
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
        other => Err(IrError::Render(format!("unknown filter `{other}`"))),
    }
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
