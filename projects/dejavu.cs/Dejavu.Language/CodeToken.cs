namespace Dejavu.Language;

/// <summary>Lexical token kinds inside <c>&lt;% ... %&gt;</c> (mirrors TS/Rust <c>CodeToken</c>).</summary>
public enum CodeTokenKind
{
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

/// <summary>A single code token with offsets relative to the code slice.</summary>
public readonly record struct CodeToken(
    CodeTokenKind Kind,
    string Text,
    bool? Value,
    int Start,
    int End);
