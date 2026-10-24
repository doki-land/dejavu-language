//! Host-native T1 AST (Rust side) that round-trips with IR.

use crate::types::{IrNode, Span};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct NativeTemplate {
    pub children: Vec<NativeNode>,
}

/// One template node and the source range it was parsed from.
///
/// Spans use the same UTF-8 byte offsets as [`IrNode`]. IR that carries no
/// spans decodes to `None`, so hosts that only feed IR still work unchanged.
#[derive(Debug, Clone, PartialEq)]
pub struct NativeNode {
    pub kind: NativeKind,
    pub span: Option<Span>,
}

impl NativeNode {
    pub fn new(kind: NativeKind, span: Option<Span>) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NativeKind {
    Text(String),
    Comment(String),
    Interpolation {
        expression: NativeExpr,
        trim: String,
        raw: bool,
    },
    If {
        test: NativeExpr,
        consequent: Vec<NativeNode>,
        else_ifs: Vec<(NativeExpr, Vec<NativeNode>)>,
        alternate: Option<Vec<NativeNode>>,
        trim: String,
    },
    For {
        item: String,
        index: Option<String>,
        iterable: NativeExpr,
        body: Vec<NativeNode>,
        trim: String,
    },
    Block {
        name: String,
        body: Vec<NativeNode>,
        trim: String,
    },
    Partial {
        params: Vec<NativeParam>,
        body: Vec<NativeNode>,
        trim: String,
    },
    Macro {
        name: String,
        params: Vec<NativeParam>,
        body: Vec<NativeNode>,
        trim: String,
    },
    Import {
        path: String,
        names: Vec<NativeImportName>,
        trim: String,
    },
    Call {
        name: String,
        arguments: Vec<NativeExpr>,
        body: Vec<NativeNode>,
        trim: String,
    },
    Extends {
        parent: NativeExpr,
        trim: String,
    },
    Include {
        path: NativeExpr,
        with: Option<NativeExpr>,
        only: bool,
        trim: String,
    },
    Super {
        trim: String,
    },
    Raw {
        value: String,
        trim: String,
    },
}

/// One declared `partial` parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct NativeParam {
    pub name: String,
    pub default: Option<NativeExpr>,
}

/// One `from ... import name as alias` binding.
#[derive(Debug, Clone, PartialEq)]
pub struct NativeImportName {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NativeExpr {
    Literal(Value),
    Identifier(String),
    Member { object: Box<NativeExpr>, property: String },
    Index { object: Box<NativeExpr>, index: Box<NativeExpr> },
    Call { callee: Box<NativeExpr>, arguments: Vec<NativeExpr> },
    Test { expression: Box<NativeExpr>, test: String, arguments: Vec<NativeExpr>, negated: bool },
    Binary { operator: String, left: Box<NativeExpr>, right: Box<NativeExpr> },
    Unary { operator: String, argument: Box<NativeExpr> },
    Pipe { expression: Box<NativeExpr>, filter: String, arguments: Vec<NativeExpr> },
    Object { entries: Vec<(String, NativeExpr)> },
}

pub fn encode_template(t: &NativeTemplate) -> IrNode {
    IrNode::Template { children: t.children.iter().map(encode_node).collect(), span: None }
}

pub fn decode_template(node: &IrNode) -> NativeTemplate {
    match node {
        IrNode::Template { children, .. } => NativeTemplate { children: children.iter().map(decode_node).collect() },
        other => NativeTemplate { children: vec![decode_node(other)] },
    }
}

fn encode_node(n: &NativeNode) -> IrNode {
    let span = n.span.clone();
    match &n.kind {
        NativeKind::Text(v) => IrNode::Text { value: v.clone(), span },
        NativeKind::Comment(v) => IrNode::Comment { value: v.clone(), span },
        NativeKind::Interpolation { expression, trim, raw } => IrNode::Interpolation {
            expression: Box::new(encode_expr(expression)),
            trim: trim.clone(),
            raw: *raw,
            span,
        },
        NativeKind::If { test, consequent, else_ifs, alternate, trim } => IrNode::StmtIf {
            test: Box::new(encode_expr(test)),
            consequent: consequent.iter().map(encode_node).collect(),
            else_ifs: else_ifs
                .iter()
                .map(|(t, body)| IrNode::StmtElseIf {
                    test: Box::new(encode_expr(t)),
                    consequent: body.iter().map(encode_node).collect(),
                    trim: "none".into(),
                    span: None,
                })
                .collect(),
            alternate: alternate.as_ref().map(|nodes| nodes.iter().map(encode_node).collect()),
            trim: trim.clone(),
            span,
        },
        NativeKind::For { item, index, iterable, body, trim } => IrNode::StmtFor {
            item: item.clone(),
            index: index.clone(),
            iterable: Box::new(encode_expr(iterable)),
            body: body.iter().map(encode_node).collect(),
            trim: trim.clone(),
            span,
        },
        NativeKind::Block { name, body, trim } => IrNode::StmtBlock {
            name: name.clone(),
            body: body.iter().map(encode_node).collect(),
            trim: trim.clone(),
            span,
        },
        NativeKind::Partial { params, body, trim } => IrNode::StmtPartial {
            params: params
                .iter()
                .map(|p| crate::types::PartialParam {
                    name: p.name.clone(),
                    default: p.default.as_ref().map(encode_expr),
                })
                .collect(),
            body: body.iter().map(encode_node).collect(),
            trim: trim.clone(),
            span,
        },
        NativeKind::Import { path, names, trim } => IrNode::StmtImport {
            path: Box::new(IrNode::ExprLiteral {
                value: serde_json::Value::String(path.clone()),
                span: span.clone(),
            }),
            names: names
                .iter()
                .map(|n| crate::types::ImportName { name: n.name.clone(), alias: n.alias.clone() })
                .collect(),
            trim: trim.clone(),
            span,
        },
        NativeKind::Macro { name, params, body, trim } => IrNode::StmtMacro {
            name: name.clone(),
            params: params
                .iter()
                .map(|p| crate::types::PartialParam {
                    name: p.name.clone(),
                    default: p.default.as_ref().map(encode_expr),
                })
                .collect(),
            body: body.iter().map(encode_node).collect(),
            trim: trim.clone(),
            span,
        },
        NativeKind::Call { name, arguments, body, trim } => IrNode::StmtCall {
            name: name.clone(),
            arguments: arguments.iter().map(encode_expr).collect(),
            body: body.iter().map(encode_node).collect(),
            trim: trim.clone(),
            span,
        },
        NativeKind::Extends { parent, trim } => {
            IrNode::StmtExtends { parent: Box::new(encode_expr(parent)), trim: trim.clone(), span }
        }
        NativeKind::Include { path, with, only, trim } => IrNode::StmtInclude {
            path: Box::new(encode_expr(path)),
            with: with.as_ref().map(|w| Box::new(encode_expr(w))),
            only: *only,
            trim: trim.clone(),
            span,
        },
        NativeKind::Super { trim } => IrNode::StmtSuper { trim: trim.clone(), span },
        NativeKind::Raw { value, trim } => IrNode::StmtRaw { value: value.clone(), trim: trim.clone(), span },
    }
}

fn encode_expr(e: &NativeExpr) -> IrNode {
    match e {
        NativeExpr::Literal(v) => IrNode::ExprLiteral { value: v.clone(), span: None },
        NativeExpr::Identifier(name) => IrNode::ExprIdentifier { name: name.clone(), span: None },
        NativeExpr::Member { object, property } => {
            IrNode::ExprMember { object: Box::new(encode_expr(object)), property: property.clone(), span: None }
        }
        NativeExpr::Index { object, index } => {
            IrNode::ExprIndex { object: Box::new(encode_expr(object)), index: Box::new(encode_expr(index)), span: None }
        }
        NativeExpr::Call { callee, arguments } => IrNode::ExprCall {
            callee: Box::new(encode_expr(callee)),
            arguments: arguments.iter().map(encode_expr).collect(),
            span: None,
        },
        NativeExpr::Test { expression, test, arguments, negated } => IrNode::ExprTest {
            expression: Box::new(encode_expr(expression)),
            test: test.clone(),
            arguments: arguments.iter().map(encode_expr).collect(),
            negated: *negated,
            span: None,
        },
        NativeExpr::Binary { operator, left, right } => IrNode::ExprBinary {
            operator: operator.clone(),
            left: Box::new(encode_expr(left)),
            right: Box::new(encode_expr(right)),
            span: None,
        },
        NativeExpr::Unary { operator, argument } => {
            IrNode::ExprUnary { operator: operator.clone(), argument: Box::new(encode_expr(argument)), span: None }
        }
        NativeExpr::Object { entries } => IrNode::ExprObject {
            entries: entries
                .iter()
                .map(|(key, value)| crate::types::ObjectEntry { key: key.clone(), value: encode_expr(value) })
                .collect(),
            span: None,
        },
        NativeExpr::Pipe { expression, filter, arguments } => IrNode::ExprPipe {
            expression: Box::new(encode_expr(expression)),
            filter: filter.clone(),
            arguments: arguments.iter().map(encode_expr).collect(),
            span: None,
        },
    }
}

fn decode_node(n: &IrNode) -> NativeNode {
    match n {
        IrNode::Text { value, span } => NativeNode::new(NativeKind::Text(value.clone()), span.clone()),
        IrNode::Comment { value, span } => NativeNode::new(NativeKind::Comment(value.clone()), span.clone()),
        IrNode::Interpolation { expression, trim, raw, span } => NativeNode::new(
            NativeKind::Interpolation { expression: decode_expr(expression), trim: trim.clone(), raw: *raw },
            span.clone(),
        ),
        IrNode::StmtIf { test, consequent, else_ifs, alternate, trim, span } => NativeNode::new(
            NativeKind::If {
                test: decode_expr(test),
                consequent: consequent.iter().map(decode_node).collect(),
                else_ifs: else_ifs
                    .iter()
                    .map(|ei| match ei {
                        IrNode::StmtElseIf { test, consequent, .. } => {
                            (decode_expr(test), consequent.iter().map(decode_node).collect())
                        }
                        _ => (NativeExpr::Literal(Value::Bool(false)), vec![]),
                    })
                    .collect(),
                alternate: alternate.as_ref().map(|nodes| nodes.iter().map(decode_node).collect()),
                trim: trim.clone(),
            },
            span.clone(),
        ),
        IrNode::StmtFor { item, index, iterable, body, trim, span } => NativeNode::new(
            NativeKind::For {
                item: item.clone(),
                index: index.clone(),
                iterable: decode_expr(iterable),
                body: body.iter().map(decode_node).collect(),
                trim: trim.clone(),
            },
            span.clone(),
        ),
        IrNode::StmtBlock { name, body, trim, span } => NativeNode::new(
            NativeKind::Block {
                name: name.clone(),
                body: body.iter().map(decode_node).collect(),
                trim: trim.clone(),
            },
            span.clone(),
        ),
        IrNode::StmtPartial { params, body, trim, span } => NativeNode::new(
            NativeKind::Partial {
                params: params
                    .iter()
                    .map(|p| NativeParam { name: p.name.clone(), default: p.default.as_ref().map(decode_expr) })
                    .collect(),
                body: body.iter().map(decode_node).collect(),
                trim: trim.clone(),
            },
            span.clone(),
        ),
        IrNode::StmtImport { path, names, trim, span } => {
            let literal = match path.as_ref() {
                IrNode::ExprLiteral { value: serde_json::Value::String(s), .. } => Some(s.clone()),
                _ => None,
            };
            match literal {
                Some(lit) => NativeNode::new(
                    NativeKind::Import {
                        path: lit,
                        names: names
                            .iter()
                            .map(|n| NativeImportName { name: n.name.clone(), alias: n.alias.clone() })
                            .collect(),
                        trim: trim.clone(),
                    },
                    span.clone(),
                ),
                // Non-literal paths cannot be represented in the native AST;
                // the renderer/codegen reject them with a structured error.
                None => NativeNode::new(
                    NativeKind::Text(format!("/* unsupported dynamic import {path:?} */")),
                    span.clone(),
                ),
            }
        }
        IrNode::StmtMacro { name, params, body, trim, span } => NativeNode::new(
            NativeKind::Macro {
                name: name.clone(),
                params: params
                    .iter()
                    .map(|p| NativeParam { name: p.name.clone(), default: p.default.as_ref().map(decode_expr) })
                    .collect(),
                body: body.iter().map(decode_node).collect(),
                trim: trim.clone(),
            },
            span.clone(),
        ),
        IrNode::StmtCall { name, arguments, body, trim, span } => NativeNode::new(
            NativeKind::Call {
                name: name.clone(),
                arguments: arguments.iter().map(decode_expr).collect(),
                body: body.iter().map(decode_node).collect(),
                trim: trim.clone(),
            },
            span.clone(),
        ),
        IrNode::StmtExtends { parent, trim, span } => NativeNode::new(
            NativeKind::Extends { parent: decode_expr(parent), trim: trim.clone() },
            span.clone(),
        ),
        IrNode::StmtInclude { path, with, only, trim, span } => NativeNode::new(
            NativeKind::Include {
                path: decode_expr(path),
                with: with.as_ref().map(|w| decode_expr(w)),
                only: *only,
                trim: trim.clone(),
            },
            span.clone(),
        ),
        IrNode::StmtSuper { trim, span } => NativeNode::new(NativeKind::Super { trim: trim.clone() }, span.clone()),
        IrNode::StmtRaw { value, trim, span } => {
            NativeNode::new(NativeKind::Raw { value: value.clone(), trim: trim.clone() }, span.clone())
        }
        other => NativeNode::new(NativeKind::Text(format!("/* unsupported {other:?} */")), None),
    }
}

fn decode_expr(n: &IrNode) -> NativeExpr {
    match n {
        IrNode::ExprLiteral { value, .. } => NativeExpr::Literal(value.clone()),
        IrNode::ExprIdentifier { name, .. } => NativeExpr::Identifier(name.clone()),
        IrNode::ExprMember { object, property, .. } => {
            NativeExpr::Member { object: Box::new(decode_expr(object)), property: property.clone() }
        }
        IrNode::ExprIndex { object, index, .. } => {
            NativeExpr::Index { object: Box::new(decode_expr(object)), index: Box::new(decode_expr(index)) }
        }
        IrNode::ExprCall { callee, arguments, .. } => {
            NativeExpr::Call { callee: Box::new(decode_expr(callee)), arguments: arguments.iter().map(decode_expr).collect() }
        }
        IrNode::ExprTest { expression, test, arguments, negated, .. } => NativeExpr::Test {
            expression: Box::new(decode_expr(expression)),
            test: test.clone(),
            arguments: arguments.iter().map(decode_expr).collect(),
            negated: *negated,
        },
        IrNode::ExprBinary { operator, left, right, .. } => NativeExpr::Binary {
            operator: operator.clone(),
            left: Box::new(decode_expr(left)),
            right: Box::new(decode_expr(right)),
        },
        IrNode::ExprUnary { operator, argument, .. } => {
            NativeExpr::Unary { operator: operator.clone(), argument: Box::new(decode_expr(argument)) }
        }
        IrNode::ExprObject { entries, .. } => NativeExpr::Object {
            entries: entries.iter().map(|e| (e.key.clone(), decode_expr(&e.value))).collect(),
        },
        IrNode::ExprPipe { expression, filter, arguments, .. } => NativeExpr::Pipe {
            expression: Box::new(decode_expr(expression)),
            filter: filter.clone(),
            arguments: arguments.iter().map(decode_expr).collect(),
        },
        _ => NativeExpr::Literal(Value::Null),
    }
}
