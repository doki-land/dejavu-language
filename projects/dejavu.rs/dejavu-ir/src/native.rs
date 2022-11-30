//! Host-native T1 AST (Rust side) that round-trips with IR.

use crate::types::IrNode;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct NativeTemplate {
    pub children: Vec<NativeNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NativeNode {
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
    Extends {
        parent: NativeExpr,
        trim: String,
    },
    Include {
        path: NativeExpr,
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

#[derive(Debug, Clone, PartialEq)]
pub enum NativeExpr {
    Literal(Value),
    Identifier(String),
    Member { object: Box<NativeExpr>, property: String },
    Index { object: Box<NativeExpr>, index: Box<NativeExpr> },
    Call { callee: Box<NativeExpr>, arguments: Vec<NativeExpr> },
    Binary { operator: String, left: Box<NativeExpr>, right: Box<NativeExpr> },
    Unary { operator: String, argument: Box<NativeExpr> },
    Pipe { expression: Box<NativeExpr>, filter: String, arguments: Vec<NativeExpr> },
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
    match n {
        NativeNode::Text(v) => IrNode::Text { value: v.clone(), span: None },
        NativeNode::Comment(v) => IrNode::Comment { value: v.clone(), span: None },
        NativeNode::Interpolation { expression, trim, raw } => {
            IrNode::Interpolation { expression: Box::new(encode_expr(expression)), trim: trim.clone(), raw: *raw, span: None }
        }
        NativeNode::If { test, consequent, else_ifs, alternate, trim } => IrNode::StmtIf {
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
            span: None,
        },
        NativeNode::For { item, index, iterable, body, trim } => IrNode::StmtFor {
            item: item.clone(),
            index: index.clone(),
            iterable: Box::new(encode_expr(iterable)),
            body: body.iter().map(encode_node).collect(),
            trim: trim.clone(),
            span: None,
        },
        NativeNode::Block { name, body, trim } => IrNode::StmtBlock {
            name: name.clone(),
            body: body.iter().map(encode_node).collect(),
            trim: trim.clone(),
            span: None,
        },
        NativeNode::Extends { parent, trim } => {
            IrNode::StmtExtends { parent: Box::new(encode_expr(parent)), trim: trim.clone(), span: None }
        }
        NativeNode::Include { path, trim } => {
            IrNode::StmtInclude { path: Box::new(encode_expr(path)), trim: trim.clone(), span: None }
        }
        NativeNode::Super { trim } => IrNode::StmtSuper { trim: trim.clone(), span: None },
        NativeNode::Raw { value, trim } => IrNode::StmtRaw { value: value.clone(), trim: trim.clone(), span: None },
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
        NativeExpr::Binary { operator, left, right } => IrNode::ExprBinary {
            operator: operator.clone(),
            left: Box::new(encode_expr(left)),
            right: Box::new(encode_expr(right)),
            span: None,
        },
        NativeExpr::Unary { operator, argument } => {
            IrNode::ExprUnary { operator: operator.clone(), argument: Box::new(encode_expr(argument)), span: None }
        }
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
        IrNode::Text { value, .. } => NativeNode::Text(value.clone()),
        IrNode::Comment { value, .. } => NativeNode::Comment(value.clone()),
        IrNode::Interpolation { expression, trim, raw, .. } => {
            NativeNode::Interpolation { expression: decode_expr(expression), trim: trim.clone(), raw: *raw }
        }
        IrNode::StmtIf { test, consequent, else_ifs, alternate, trim, .. } => NativeNode::If {
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
        IrNode::StmtFor { item, index, iterable, body, trim, .. } => NativeNode::For {
            item: item.clone(),
            index: index.clone(),
            iterable: decode_expr(iterable),
            body: body.iter().map(decode_node).collect(),
            trim: trim.clone(),
        },
        IrNode::StmtBlock { name, body, trim, .. } => {
            NativeNode::Block { name: name.clone(), body: body.iter().map(decode_node).collect(), trim: trim.clone() }
        }
        IrNode::StmtExtends { parent, trim, .. } => NativeNode::Extends { parent: decode_expr(parent), trim: trim.clone() },
        IrNode::StmtInclude { path, trim, .. } => NativeNode::Include { path: decode_expr(path), trim: trim.clone() },
        IrNode::StmtSuper { trim, .. } => NativeNode::Super { trim: trim.clone() },
        IrNode::StmtRaw { value, trim, .. } => NativeNode::Raw { value: value.clone(), trim: trim.clone() },
        other => NativeNode::Text(format!("/* unsupported {:?} */", other)),
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
        IrNode::ExprBinary { operator, left, right, .. } => NativeExpr::Binary {
            operator: operator.clone(),
            left: Box::new(decode_expr(left)),
            right: Box::new(decode_expr(right)),
        },
        IrNode::ExprUnary { operator, argument, .. } => {
            NativeExpr::Unary { operator: operator.clone(), argument: Box::new(decode_expr(argument)) }
        }
        IrNode::ExprPipe { expression, filter, arguments, .. } => NativeExpr::Pipe {
            expression: Box::new(decode_expr(expression)),
            filter: filter.clone(),
            arguments: arguments.iter().map(decode_expr).collect(),
        },
        _ => NativeExpr::Literal(Value::Null),
    }
}
