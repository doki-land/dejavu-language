use serde::{Deserialize, Serialize};

/// Top-level IR document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IrDocument {
    pub ir_version: String,
    pub language: Language,
    pub body: IrNode,
}

/// Language + delimiter configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    pub syntax_mode: String,
    pub template: TemplateConfig,
}

/// Template delimiter / feature flags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateConfig {
    pub code_start: String,
    pub code_end: String,
    pub comment_start: String,
    pub comment_end: String,
    pub support_filter_pipe: bool,
    pub legacy_for: bool,
}

/// Default T1 language config.
pub fn default_language() -> Language {
    Language {
        syntax_mode: "template".into(),
        template: TemplateConfig {
            code_start: "<%".into(),
            code_end: "%>".into(),
            comment_start: "<#".into(),
            comment_end: "#>".into(),
            support_filter_pipe: true,
            legacy_for: false,
        },
    }
}

/// IR node (structural / statement / expression).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IrNode {
    Template {
        children: Vec<IrNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    Text {
        value: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    Comment {
        value: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    Interpolation {
        expression: Box<IrNode>,
        trim: String,
        #[serde(default, skip_serializing_if = "is_false")]
        raw: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Stmt.If")]
    StmtIf {
        test: Box<IrNode>,
        consequent: Vec<IrNode>,
        #[serde(rename = "elseIfs")]
        else_ifs: Vec<IrNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        alternate: Option<Vec<IrNode>>,
        trim: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Stmt.Partial")]
    StmtPartial {
        params: Vec<PartialParam>,
        body: Vec<IrNode>,
        trim: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    /// `macro name(params)` — a named template-local component. Invoked through
    /// a bare-identifier [`IrNode::ExprCall`]; its body sees its parameters
    /// only, never the caller scope.
    #[serde(rename = "Stmt.Macro")]
    StmtMacro {
        name: String,
        params: Vec<PartialParam>,
        body: Vec<IrNode>,
        trim: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Stmt.ElseIf")]
    StmtElseIf {
        test: Box<IrNode>,
        consequent: Vec<IrNode>,
        trim: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Stmt.For")]
    StmtFor {
        item: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<String>,
        iterable: Box<IrNode>,
        body: Vec<IrNode>,
        trim: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Stmt.Block")]
    StmtBlock {
        name: String,
        body: Vec<IrNode>,
        trim: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Stmt.Extends")]
    StmtExtends {
        parent: Box<IrNode>,
        trim: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Stmt.Include")]
    StmtInclude {
        path: Box<IrNode>,
        /// Explicit child context merged over the inherited one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        with: Option<Box<IrNode>>,
        /// `only` / `without context`: the child sees no inherited context.
        #[serde(default, skip_serializing_if = "is_false")]
        only: bool,
        trim: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Stmt.Super")]
    StmtSuper {
        trim: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Stmt.Raw")]
    StmtRaw {
        value: String,
        trim: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Expr.Literal")]
    ExprLiteral {
        value: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Expr.Identifier")]
    ExprIdentifier {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Expr.Member")]
    ExprMember {
        object: Box<IrNode>,
        property: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Expr.Index")]
    ExprIndex {
        object: Box<IrNode>,
        index: Box<IrNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Expr.Call")]
    ExprCall {
        callee: Box<IrNode>,
        arguments: Vec<IrNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    /// `expression is [not] test(args)` — predicate from the `test` namespace.
    #[serde(rename = "Expr.Test")]
    ExprTest {
        expression: Box<IrNode>,
        test: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        arguments: Vec<IrNode>,
        #[serde(default, skip_serializing_if = "is_false")]
        negated: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Expr.Binary")]
    ExprBinary {
        operator: String,
        left: Box<IrNode>,
        right: Box<IrNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Expr.Unary")]
    ExprUnary {
        operator: String,
        argument: Box<IrNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Expr.Object")]
    ExprObject {
        entries: Vec<ObjectEntry>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    #[serde(rename = "Expr.Pipe")]
    ExprPipe {
        expression: Box<IrNode>,
        filter: String,
        arguments: Vec<IrNode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
}

/// One `partial` / `macro` parameter: name plus an optional literal default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PartialParam {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<IrNode>,
}

/// One `key: value` entry of an object literal expression.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectEntry {
    pub key: String,
    pub value: IrNode,
}

/// Source span in UTF-8 byte offsets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

fn is_false(v: &bool) -> bool {
    !*v
}
