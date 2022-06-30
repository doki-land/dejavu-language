"""Lexical token kinds inside `<% ... %>` (mirrors TS/Rust CodeToken)."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

CodeTokenKind = Literal[
    "CodeEnd",
    "PipeOp",
    "OrOr",
    "AndAnd",
    "EqEq",
    "NotEq",
    "LessEq",
    "GreaterEq",
    "Less",
    "Greater",
    "Plus",
    "Minus",
    "Star",
    "Slash",
    "Percent",
    "Bang",
    "Dot",
    "Comma",
    "LParen",
    "RParen",
    "LBracket",
    "RBracket",
    "Bool",
    "Null",
    "In",
    "Ident",
    "Number",
    "String",
]


@dataclass(slots=True)
class CodeToken:
    kind: CodeTokenKind
    """Lexeme text (idents, numbers, string contents, bool literal text)."""
    text: str
    start: int
    end: int
    value: bool | None = None
