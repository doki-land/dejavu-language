"""Hand-written lexer for code inside `<% ... %>`. No `re` module."""

from __future__ import annotations

from dejavu_language.error import ParseError
from dejavu_language.token import CodeToken, CodeTokenKind

_MAP2: dict[str, CodeTokenKind] = {
    "%>": "CodeEnd",
    "|>": "PipeOp",
    "||": "OrOr",
    "&&": "AndAnd",
    "==": "EqEq",
    "!=": "NotEq",
    "<=": "LessEq",
    ">=": "GreaterEq",
}

_MAP1: dict[str, CodeTokenKind] = {
    "<": "Less",
    ">": "Greater",
    "+": "Plus",
    "-": "Minus",
    "*": "Star",
    "/": "Slash",
    "%": "Percent",
    "!": "Bang",
    ".": "Dot",
    ",": "Comma",
    "(": "LParen",
    ")": "RParen",
    "[": "LBracket",
    "]": "RBracket",
}


def lex_code(
    input_str: str,
    *,
    source: str | None = None,
    file: str = "template.dejavu",
    base: int = 0,
) -> list[CodeToken]:
    _ = source if source is not None else input_str
    tokens: list[CodeToken] = []
    i = 0
    n = len(input_str)

    def push(
        kind: CodeTokenKind,
        start: int,
        end: int,
        text: str = "",
        value: bool | None = None,
    ) -> None:
        tokens.append(CodeToken(kind=kind, text=text, start=start, end=end, value=value))

    def fail(start: int, length: int, message: str) -> None:
        raise ParseError(
            message,
            file=file,
            start=base + start,
            length=length,
            label="bad token",
        )

    while i < n:
        c = ord(input_str[i])
        # whitespace
        if c in (0x20, 0x09, 0x0A, 0x0D):
            i += 1
            continue

        # two-char ops
        if i + 1 < n:
            two = input_str[i : i + 2]
            kind2 = _MAP2.get(two)
            if kind2 is not None:
                push(kind2, i, i + 2, two)
                i += 2
                continue

        one = input_str[i]
        kind1 = _MAP1.get(one)
        if kind1 is not None:
            push(kind1, i, i + 1, one)
            i += 1
            continue

        # string
        if one in "\"'":
            quote = one
            start = i
            i += 1
            text_parts: list[str] = []
            closed = False
            while i < n:
                ch = input_str[i]
                if ch == "\\":
                    if i + 1 >= n:
                        fail(start, i - start + 1, "unterminated string escape")
                    text_parts.append(input_str[i + 1])
                    i += 2
                    continue
                if ch == quote:
                    i += 1
                    push("String", start, i, "".join(text_parts))
                    closed = True
                    break
                text_parts.append(ch)
                i += 1
            if not closed:
                fail(start, max(1, i - start), "unterminated string")
            continue

        # number
        if 0x30 <= c <= 0x39:
            start = i
            i += 1
            while i < n:
                d = ord(input_str[i])
                if 0x30 <= d <= 0x39:
                    i += 1
                else:
                    break
            if i < n and input_str[i] == ".":
                i += 1
                while i < n:
                    d = ord(input_str[i])
                    if 0x30 <= d <= 0x39:
                        i += 1
                    else:
                        break
            push("Number", start, i, input_str[start:i])
            continue

        # ident / keywords
        if _is_ident_start(c):
            start = i
            i += 1
            while i < n and _is_ident_continue(ord(input_str[i])):
                i += 1
            text = input_str[start:i]
            if text == "true":
                push("Bool", start, i, text, True)
            elif text == "false":
                push("Bool", start, i, text, False)
            elif text == "null":
                push("Null", start, i, text, None)
            elif text == "in":
                push("In", start, i, text)
            else:
                push("Ident", start, i, text)
            continue

        fail(i, 1, f"invalid token in expression ({one!r})")

    return tokens


def _is_ident_start(c: int) -> bool:
    return (0x41 <= c <= 0x5A) or (0x61 <= c <= 0x7A) or c == 0x5F


def _is_ident_continue(c: int) -> bool:
    return _is_ident_start(c) or (0x30 <= c <= 0x39)
