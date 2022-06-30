"""Pratt-style expression parser over a CodeToken stream."""

from __future__ import annotations

from typing import Any

from dejavu_language.error import ParseError
from dejavu_language.lexer import lex_code
from dejavu_language.token import CodeToken, CodeTokenKind


class ExprParser:
    def __init__(self, source: str, file: str, base: int, input_str: str) -> None:
        self.source = source
        self.file = file
        self.base = base
        self.tokens = lex_code(input_str, source=source, file=file, base=base)
        for t in self.tokens:
            if t.kind == "CodeEnd":
                raise ParseError(
                    "unexpected `%>` inside expression",
                    file=file,
                    start=base + t.start,
                    length=t.end - t.start,
                )
        self.pos = 0

    def parse(self) -> dict[str, Any]:
        expr = self._parse_pipe()
        if self.pos != len(self.tokens):
            span = self._peek_span()
            raise ParseError(
                "trailing input in expression",
                file=self.file,
                start=span[0],
                length=span[1],
                label="unexpected",
            )
        return expr

    def _peek(self) -> CodeToken | None:
        if self.pos < len(self.tokens):
            return self.tokens[self.pos]
        return None

    def _peek_kind(self) -> CodeTokenKind | None:
        t = self._peek()
        return t.kind if t else None

    def _peek_span(self) -> tuple[int, int]:
        t = self._peek()
        if t:
            return self.base + t.start, max(1, t.end - t.start)
        last = self.tokens[-1] if self.tokens else None
        end = self.base + (last.end if last else 0)
        return end, 1

    def _bump(self) -> CodeToken | None:
        t = self._peek()
        if t is not None:
            self.pos += 1
        return t

    def _expect_ident(self) -> str:
        t = self._bump()
        if t is not None and t.kind == "Ident":
            return t.text
        span = self._peek_span()
        raise ParseError(
            "expected identifier",
            file=self.file,
            start=span[0],
            length=span[1],
            label="expected ident",
        )

    def _parse_pipe(self) -> dict[str, Any]:
        left = self._parse_or()
        while self._peek_kind() == "PipeOp":
            self._bump()
            filt = self._expect_ident()
            args: list[dict[str, Any]] = []
            if self._peek_kind() == "LParen":
                self._bump()
                if self._peek_kind() != "RParen":
                    while True:
                        args.append(self._parse_pipe())
                        if self._peek_kind() == "Comma":
                            self._bump()
                            continue
                        break
                closed = self._bump()
                if closed is None or closed.kind != "RParen":
                    span = self._peek_span()
                    raise ParseError(
                        "expected `)` after filter arguments",
                        file=self.file,
                        start=span[0],
                        length=span[1],
                    )
            left = {
                "type": "Expr.Pipe",
                "expression": left,
                "filter": filt,
                "arguments": args,
            }
        return left

    def _parse_or(self) -> dict[str, Any]:
        left = self._parse_and()
        while self._peek_kind() == "OrOr":
            self._bump()
            left = {
                "type": "Expr.Binary",
                "operator": "||",
                "left": left,
                "right": self._parse_and(),
            }
        return left

    def _parse_and(self) -> dict[str, Any]:
        left = self._parse_cmp()
        while self._peek_kind() == "AndAnd":
            self._bump()
            left = {
                "type": "Expr.Binary",
                "operator": "&&",
                "left": left,
                "right": self._parse_cmp(),
            }
        return left

    def _parse_cmp(self) -> dict[str, Any]:
        left = self._parse_add()
        kind = self._peek_kind()
        op_map: dict[CodeTokenKind, str] = {
            "EqEq": "==",
            "NotEq": "!=",
            "LessEq": "<=",
            "GreaterEq": ">=",
            "Less": "<",
            "Greater": ">",
            "In": "in",
        }
        op = op_map.get(kind) if kind else None
        if op is not None:
            self._bump()
            return {
                "type": "Expr.Binary",
                "operator": op,
                "left": left,
                "right": self._parse_add(),
            }
        return left

    def _parse_add(self) -> dict[str, Any]:
        left = self._parse_mul()
        while True:
            kind = self._peek_kind()
            if kind in ("Plus", "Minus"):
                op = "+" if kind == "Plus" else "-"
                self._bump()
                left = {
                    "type": "Expr.Binary",
                    "operator": op,
                    "left": left,
                    "right": self._parse_mul(),
                }
            else:
                break
        return left

    def _parse_mul(self) -> dict[str, Any]:
        left = self._parse_unary()
        while True:
            kind = self._peek_kind()
            if kind in ("Star", "Slash", "Percent"):
                op = "*" if kind == "Star" else "/" if kind == "Slash" else "%"
                self._bump()
                left = {
                    "type": "Expr.Binary",
                    "operator": op,
                    "left": left,
                    "right": self._parse_unary(),
                }
            else:
                break
        return left

    def _parse_unary(self) -> dict[str, Any]:
        kind = self._peek_kind()
        if kind in ("Bang", "Minus", "Plus"):
            op = "!" if kind == "Bang" else "-" if kind == "Minus" else "+"
            self._bump()
            return {
                "type": "Expr.Unary",
                "operator": op,
                "argument": self._parse_unary(),
            }
        return self._parse_postfix()

    def _parse_postfix(self) -> dict[str, Any]:
        left = self._parse_primary()
        while True:
            kind = self._peek_kind()
            if kind == "Dot":
                self._bump()
                left = {
                    "type": "Expr.Member",
                    "object": left,
                    "property": self._expect_ident(),
                }
            elif kind == "LBracket":
                self._bump()
                index = self._parse_pipe()
                closed = self._bump()
                if closed is None or closed.kind != "RBracket":
                    span = self._peek_span()
                    raise ParseError(
                        "expected `]`",
                        file=self.file,
                        start=span[0],
                        length=span[1],
                    )
                left = {"type": "Expr.Index", "object": left, "index": index}
            elif kind == "LParen":
                self._bump()
                args: list[dict[str, Any]] = []
                if self._peek_kind() != "RParen":
                    while True:
                        args.append(self._parse_pipe())
                        if self._peek_kind() == "Comma":
                            self._bump()
                            continue
                        break
                closed = self._bump()
                if closed is None or closed.kind != "RParen":
                    span = self._peek_span()
                    raise ParseError(
                        "expected `)`",
                        file=self.file,
                        start=span[0],
                        length=span[1],
                    )
                left = {"type": "Expr.Call", "callee": left, "arguments": args}
            else:
                break
        return left

    def _parse_primary(self) -> dict[str, Any]:
        t = self._bump()
        if t is None:
            span = self._peek_span()
            raise ParseError(
                "unexpected end of expression",
                file=self.file,
                start=span[0],
                length=span[1],
            )
        if t.kind == "String":
            return {"type": "Expr.Literal", "value": t.text}
        if t.kind == "Bool":
            return {"type": "Expr.Literal", "value": bool(t.value)}
        if t.kind == "Null":
            return {"type": "Expr.Literal", "value": None}
        if t.kind == "Number":
            try:
                if "." in t.text:
                    num: int | float = float(t.text)
                else:
                    num = int(t.text)
            except ValueError as e:
                raise ParseError(
                    f"invalid number `{t.text}`",
                    file=self.file,
                    start=self.base + t.start,
                    length=t.end - t.start,
                ) from e
            return {"type": "Expr.Literal", "value": num}
        if t.kind == "Ident":
            return {"type": "Expr.Identifier", "name": t.text}
        if t.kind == "LParen":
            e = self._parse_pipe()
            closed = self._bump()
            if closed is None or closed.kind != "RParen":
                span = self._peek_span()
                raise ParseError(
                    "expected `)`",
                    file=self.file,
                    start=span[0],
                    length=span[1],
                )
            return e
        raise ParseError(
            "unexpected token in expression",
            file=self.file,
            start=self.base + t.start,
            length=max(1, t.end - t.start),
        )


def parse_expr(
    input_str: str,
    *,
    source: str | None = None,
    file: str = "template.dejavu",
    base: int = 0,
) -> dict[str, Any]:
    src = source if source is not None else input_str
    return ExprParser(src, file, base, input_str).parse()
