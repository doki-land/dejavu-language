use serde::{Deserialize, Deserializer, Serialize};

/// The only IR contract version accepted on the wire (`/v0` namespace).
/// Deserialization of any other `irVersion` is rejected explicitly.
pub const IR_VERSION: &str = "v0";

/// Top-level IR document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IrDocument {
    #[serde(deserialize_with = "deserialize_ir_version")]
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
    /// `from "<path>" import a, b as c` — static macro import.
    ///
    /// The path must be a string literal so the dependency graph is fully
    /// static for both JIT loading and AOT catalog emission.
    #[serde(rename = "Stmt.Import")]
    StmtImport {
        path: Box<IrNode>,
        names: Vec<ImportName>,
        trim: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    /// `call name(args)` … `end call` — invokes a macro with a caller block.
    ///
    /// The block is exposed to the macro body as `caller()` and renders in the
    /// calling template's scope, not the macro's. `params` are the caller
    /// block's formal parameters, declared as `call(a, b) name(args)`.
    #[serde(rename = "Stmt.Call")]
    StmtCall {
        name: String,
        arguments: Vec<IrNode>,
        #[serde(default)]
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
    /// `loop item in iterable` — body iteration.
    ///
    /// Loop metadata (`index`, `index0`, `first`, `last`, `length`, `depth`)
    /// is bound implicitly as the `forloop` object inside every body
    /// iteration; it is not part of this node's shape.
    #[serde(rename = "Stmt.For")]
    StmtFor {
        item: String,
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
    /// `break` — terminates the nearest enclosing loop in the same lexical unit.
    #[serde(rename = "Stmt.Break")]
    StmtBreak {
        trim: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<Span>,
    },
    /// `continue` — skips the rest of the bound loop's current iteration.
    #[serde(rename = "Stmt.Continue")]
    StmtContinue {
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

/// One imported name: the macro declared by the target template plus the
/// local name it is bound to (`as` alias when present).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportName {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

impl ImportName {
    /// Local name this import binds in the importing template.
    pub fn local(&self) -> &str {
        self.alias.as_deref().unwrap_or(&self.name)
    }
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

fn deserialize_ir_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == IR_VERSION {
        Ok(value)
    } else {
        Err(serde::de::Error::custom(format!("unsupported irVersion {value:?}, expected {IR_VERSION:?}")))
    }
}
