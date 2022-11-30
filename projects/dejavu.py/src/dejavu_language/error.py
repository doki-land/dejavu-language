"""Span-aware parse failure."""

from __future__ import annotations


class ParseError(Exception):
    def __init__(
        self,
        message: str,
        *,
        file: str = "template.dejavu",
        start: int,
        length: int | None = None,
        label: str = "here",
    ) -> None:
        super().__init__(message)
        self.file = file
        self.start = start
        self.length = max(1, 1 if length is None else length)
        self.label = label
