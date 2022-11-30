"""Dejavu T1 language frontend — hand-written lexer + recursive-descent parser."""

from __future__ import annotations

from dejavu_language.error import ParseError
from dejavu_language.expr import parse_expr
from dejavu_language.lexer import lex_code
from dejavu_language.parser import parse_to_ir

__all__ = ["parse_to_ir", "parse_expr", "ParseError", "lex_code"]
