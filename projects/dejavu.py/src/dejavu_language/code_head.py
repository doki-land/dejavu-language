"""Classify a code block from its token stream (no string-prefix hacks)."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal

from dejavu_language.error import ParseError
from dejavu_language.token import CodeToken


@dataclass(slots=True)
class CodeHeadIf:
    kind: Literal["if"]
    test_slice: str
    test_abs: int


@dataclass(slots=True)
class CodeHeadLoop:
    kind: Literal["loop"]
    item: str
    iter_slice: str
    iter_abs: int


@dataclass(slots=True)
class CodeHeadElseIf:
    kind: Literal["else_if"]
    test_slice: str
    test_abs: int


@dataclass(slots=True)
class CodeHeadElse:
    kind: Literal["else"]


@dataclass(slots=True)
class CodeHeadEndIf:
    kind: Literal["end_if"]


@dataclass(slots=True)
class CodeHeadEndLoop:
    kind: Literal["end_loop"]


@dataclass(slots=True)
class CodeHeadExpr:
    kind: Literal["expr"]
    expr_slice: str
    expr_abs: int


CodeHead = (
    CodeHeadIf
    | CodeHeadLoop
    | CodeHeadElseIf
    | CodeHeadElse
    | CodeHeadEndIf
    | CodeHeadEndLoop
    | CodeHeadExpr
)

CodeHeadKind = Literal["if", "loop", "else_if", "else", "end_if", "end_loop", "expr"]


def slice_from_tokens(
    content: str,
    content_base: int,
    tokens: list[CodeToken],
) -> tuple[str, int]:
    if not tokens:
        return "", content_base
    start = tokens[0].start
    end = tokens[-1].end
    return content[start:end], content_base + start


def classify_code(content: str, content_base: int, tokens: list[CodeToken]) -> CodeHead:
    if not tokens:
        return CodeHeadExpr(kind="expr", expr_slice="", expr_abs=content_base)

    t0 = tokens[0]
    if t0.kind == "Ident" and t0.text == "if":
        slice_, abs_ = slice_from_tokens(content, content_base, tokens[1:])
        return CodeHeadIf(kind="if", test_slice=slice_, test_abs=abs_)

    if t0.kind == "Ident" and t0.text == "loop":
        if len(tokens) < 2 or tokens[1].kind != "Ident":
            raise ParseError(
                "loop requires item identifier",
                start=content_base + t0.start,
                length=max(1, (tokens[-1].end if tokens else t0.end) - t0.start),
            )
        if len(tokens) < 3 or tokens[2].kind != "In":
            raise ParseError(
                "loop requires `in`",
                start=content_base + t0.start,
                length=max(1, (tokens[-1].end if tokens else t0.end) - t0.start),
                label="expected `item in iterable`",
            )
        item = tokens[1].text
        slice_, abs_ = slice_from_tokens(content, content_base, tokens[3:])
        return CodeHeadLoop(kind="loop", item=item, iter_slice=slice_, iter_abs=abs_)

    if t0.kind == "Ident" and t0.text == "else":
        if len(tokens) > 1 and tokens[1].kind == "Ident" and tokens[1].text == "if":
            slice_, abs_ = slice_from_tokens(content, content_base, tokens[2:])
            return CodeHeadElseIf(kind="else_if", test_slice=slice_, test_abs=abs_)
        if len(tokens) == 1:
            return CodeHeadElse(kind="else")
        raise ParseError(
            "unexpected tokens after `else`",
            start=content_base + tokens[1].start,
            length=1,
        )

    if t0.kind == "Ident" and t0.text == "end":
        if (
            len(tokens) == 2
            and tokens[1].kind == "Ident"
            and tokens[1].text == "if"
        ):
            return CodeHeadEndIf(kind="end_if")
        if (
            len(tokens) == 2
            and tokens[1].kind == "Ident"
            and tokens[1].text == "loop"
        ):
            return CodeHeadEndLoop(kind="end_loop")
        raise ParseError(
            "expected `end if` or `end loop`",
            start=content_base + t0.start,
            length=max(1, (tokens[-1].end if tokens else t0.end) - t0.start),
        )

    slice_, abs_ = slice_from_tokens(content, content_base, tokens)
    return CodeHeadExpr(kind="expr", expr_slice=slice_, expr_abs=abs_)
