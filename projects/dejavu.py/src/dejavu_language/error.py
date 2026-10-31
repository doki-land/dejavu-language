"""Span-aware parse failure."""

from __future__ import annotations

from typing import Any


class ParseError(Exception):
    def __init__(
        self,
        message: str,
        *,
        file: str = "template.dejavu",
        start: int,
        length: int | None = None,
        label: str = "here",
        code: str | None = None,
        diagnostic_args: dict[str, Any] | None = None,
    ) -> None:
        super().__init__(message)
        self.file = file
        self.start = start
        self.length = max(1, 1 if length is None else length)
        self.label = label
        self.code = code
        self.diagnostic_args = diagnostic_args or {}
