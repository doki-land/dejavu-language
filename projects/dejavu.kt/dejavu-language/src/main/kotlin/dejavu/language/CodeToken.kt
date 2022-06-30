package dejavu.language

/** Lexical token kinds inside `<% ... %>` (mirrors TS / Rust `CodeToken`). */
enum class CodeTokenKind {
    CodeEnd,
    PipeOp,
    OrOr,
    AndAnd,
    EqEq,
    NotEq,
    LessEq,
    GreaterEq,
    Less,
    Greater,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Dot,
    Comma,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Bool,
    Null,
    In,
    Ident,
    Number,
    String,
}

data class CodeToken(
    val kind: CodeTokenKind,
    /** Lexeme text (idents, numbers, string contents, bool literal text). */
    val text: String = "",
    val value: Boolean? = null,
    /** Offset within the code slice (not absolute source). */
    val start: Int,
    val end: Int,
)
