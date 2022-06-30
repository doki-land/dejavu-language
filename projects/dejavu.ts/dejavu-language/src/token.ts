/** Lexical token kinds inside `<% ... %>` (mirrors Rust `CodeToken`). */
export type CodeTokenKind =
    | "CodeEnd"
    | "PipeOp"
    | "OrOr"
    | "AndAnd"
    | "EqEq"
    | "NotEq"
    | "LessEq"
    | "GreaterEq"
    | "Less"
    | "Greater"
    | "Plus"
    | "Minus"
    | "Star"
    | "Slash"
    | "Percent"
    | "Bang"
    | "Dot"
    | "Comma"
    | "Colon"
    | "LParen"
    | "RParen"
    | "LBracket"
    | "RBracket"
    | "Bool"
    | "Null"
    | "In"
    | "Ident"
    | "Number"
    | "String";

export type CodeToken = {
    kind: CodeTokenKind;
    /** Lexeme text (idents, numbers, string contents, bool literal text). */
    text: string;
    value?: boolean | null;
    /** Offset within the code slice (not absolute source). */
    start: number;
    end: number;
};
