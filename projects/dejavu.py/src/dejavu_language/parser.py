"""Recursive-descent T1 template parser producing Dejavu IR."""

from __future__ import annotations

from typing import Any, Literal

from dejavu_types import DEFAULT_LANGUAGE

from dejavu_language.code_head import (
    CodeHeadExpr,
    CodeHeadIf,
    CodeHeadKind,
    CodeHeadLoop,
    classify_code,
)
from dejavu_language.error import ParseError
from dejavu_language.expr import parse_expr
from dejavu_language.lexer import lex_code

TrimMode = Literal["none", "ws", "nl", "ws_nl", "all"]


def parse_to_ir(source: str, file: str = "template.dejavu") -> dict[str, Any]:
    return {
        "irVersion": "1.0",
        "language": {
            "syntaxMode": DEFAULT_LANGUAGE["syntaxMode"],
            "template": dict(DEFAULT_LANGUAGE["template"]),
        },
        "body": {"type": "Template", "children": TemplateParser(source, file).parse()},
    }


class TemplateParser:
    def __init__(self, source: str, file: str) -> None:
        self.source = source
        self.file = file

    def parse(self) -> list[dict[str, Any]]:
        children, _ = self._parse_body(0, [])
        return children

    def _parse_body(
        self, i: int, stop: list[CodeHeadKind]
    ) -> tuple[list[dict[str, Any]], int]:
        children: list[dict[str, Any]] = []
        source = self.source

        while i < len(source):
            if source.startswith("<%", i) and stop:
                block = self._read_code_block(i)
                head = classify_code(block["content"], block["content_base"], block["tokens"])
                if head.kind in stop:
                    return children, i

            if source.startswith("<#", i):
                end = self._find_delimiter(i + 2, "#>")
                if end < 0:
                    raise ParseError(
                        "unclosed comment",
                        file=self.file,
                        start=i,
                        length=2,
                        label="comment starts here",
                    )
                children.append({"type": "Comment", "value": source[i + 2 : end]})
                i = end + 2
                continue

            if source.startswith("<%!", i):
                children.append({"type": "Text", "value": "<%"})
                i += 3
                continue

            if source.startswith("<%", i):
                open_pos = i
                block = self._read_code_block(i)
                i = block["next"]
                head = classify_code(block["content"], block["content_base"], block["tokens"])

                if stop and head.kind in stop:
                    return children, open_pos

                if head.kind == "if":
                    assert isinstance(head, CodeHeadIf)
                    node, i = self._parse_if(i, head, block["trim"])
                    children.append(node)
                elif head.kind == "loop":
                    assert isinstance(head, CodeHeadLoop)
                    node, i = self._parse_loop(i, head, block["trim"])
                    children.append(node)
                elif head.kind in ("end_if", "end_loop", "else", "else_if"):
                    if not stop:
                        raise ParseError(
                            f"unexpected control `{head.kind}`",
                            file=self.file,
                            start=block["content_base"],
                            length=1,
                        )
                    return children, open_pos
                else:
                    assert isinstance(head, CodeHeadExpr)
                    children.append(
                        {
                            "type": "Interpolation",
                            "expression": parse_expr(
                                head.expr_slice,
                                source=source,
                                file=self.file,
                                base=head.expr_abs,
                            ),
                            "trim": block["trim"],
                        }
                    )
                continue

            nxt = self._next_markup(i)
            if nxt is None:
                children.append({"type": "Text", "value": source[i:]})
                break
            if nxt > i:
                children.append({"type": "Text", "value": source[i:nxt]})
                i = nxt
            else:
                i += 1

        return children, i

    def _parse_if(
        self, i: int, head: CodeHeadIf, trim: TrimMode
    ) -> tuple[dict[str, Any], int]:
        test = parse_expr(
            head.test_slice,
            source=self.source,
            file=self.file,
            base=head.test_abs,
        )
        consequent, i = self._parse_body(i, ["else_if", "else", "end_if"])
        else_ifs: list[dict[str, Any]] = []
        alternate: list[dict[str, Any]] | None = None

        while True:
            block = self._read_code_block(i)
            h = classify_code(block["content"], block["content_base"], block["tokens"])
            if h.kind == "else_if":
                i = block["next"]
                t = parse_expr(
                    h.test_slice,
                    source=self.source,
                    file=self.file,
                    base=h.test_abs,
                )
                body, i = self._parse_body(i, ["else_if", "else", "end_if"])
                else_ifs.append(
                    {
                        "type": "Stmt.ElseIf",
                        "test": t,
                        "consequent": body,
                        "trim": "none",
                    }
                )
            elif h.kind == "else":
                i = block["next"]
                body, i = self._parse_body(i, ["end_if"])
                alternate = body
                end = self._read_code_block(i)
                if (
                    classify_code(end["content"], end["content_base"], end["tokens"]).kind
                    != "end_if"
                ):
                    raise ParseError(
                        "expected `end if`",
                        file=self.file,
                        start=end["content_base"],
                        length=1,
                    )
                i = end["next"]
                break
            elif h.kind == "end_if":
                i = block["next"]
                break
            else:
                raise ParseError(
                    f"expected if closer, got `{h.kind}`",
                    file=self.file,
                    start=block["content_base"],
                    length=1,
                )

        node: dict[str, Any] = {
            "type": "Stmt.If",
            "test": test,
            "consequent": consequent,
            "elseIfs": else_ifs,
            "trim": trim,
        }
        if alternate is not None:
            node["alternate"] = alternate
        return node, i

    def _parse_loop(
        self, i: int, head: CodeHeadLoop, trim: TrimMode
    ) -> tuple[dict[str, Any], int]:
        iterable = parse_expr(
            head.iter_slice,
            source=self.source,
            file=self.file,
            base=head.iter_abs,
        )
        body, i = self._parse_body(i, ["end_loop"])
        end = self._read_code_block(i)
        if (
            classify_code(end["content"], end["content_base"], end["tokens"]).kind
            != "end_loop"
        ):
            raise ParseError(
                "expected `end loop`",
                file=self.file,
                start=end["content_base"],
                length=1,
            )
        return (
            {
                "type": "Stmt.For",
                "item": head.item,
                "iterable": iterable,
                "body": body,
                "trim": trim,
            },
            end["next"],
        )

    def _read_code_block(self, i: int) -> dict[str, Any]:
        source = self.source
        if not source.startswith("<%", i):
            raise ParseError(
                "expected code open `<%`",
                file=self.file,
                start=i,
                length=1,
            )
        j = i + 2
        trim: TrimMode = "none"
        if j < len(source) and source[j] in "._-~=":
            mod = source[j]
            trim = {
                ".": "none",
                "_": "ws",
                "-": "nl",
                "~": "ws_nl",
                "=": "all",
            }[mod]
            j += 1
        end = self._find_delimiter(j, "%>")
        if end < 0:
            raise ParseError(
                "unclosed code block",
                file=self.file,
                start=i,
                length=2,
                label="opens here",
            )
        content_base = j
        while content_base < end:
            c = ord(source[content_base])
            if c in (0x20, 0x09, 0x0A, 0x0D):
                content_base += 1
            else:
                break
        content_end = end
        while content_end > content_base:
            c = ord(source[content_end - 1])
            if c in (0x20, 0x09, 0x0A, 0x0D):
                content_end -= 1
            else:
                break
        content = source[content_base:content_end]
        tokens = [
            t
            for t in lex_code(content, source=source, file=self.file, base=content_base)
            if t.kind != "CodeEnd"
        ]
        return {
            "tokens": tokens,
            "content": content,
            "content_base": content_base,
            "trim": trim,
            "next": end + 2,
        }

    def _find_delimiter(self, from_pos: int, delim: str) -> int:
        source = self.source
        d0 = ord(delim[0])
        dlen = len(delim)
        i = from_pos
        limit = len(source) - dlen + 1
        while i < limit:
            if ord(source[i]) != d0:
                i += 1
                continue
            ok = True
            for k in range(1, dlen):
                if ord(source[i + k]) != ord(delim[k]):
                    ok = False
                    break
            if ok:
                return i
            i += 1
        return -1

    def _next_markup(self, from_pos: int) -> int | None:
        source = self.source
        i = from_pos
        n = len(source)
        while i + 1 < n:
            if ord(source[i]) != 0x3C:  # <
                i += 1
                continue
            nxt = ord(source[i + 1])
            if nxt in (0x25, 0x23):  # % or #
                return i
            i += 1
        return None
