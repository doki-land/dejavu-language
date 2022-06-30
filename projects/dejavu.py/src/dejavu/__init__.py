"""
Public Python surface: ``from dejavu import Dejavu, parse, render``.

Internal modules (``dejavu_language``, ``dejavu_types``, ``dejavu_engine``) are
implementation details — application code should import **``dejavu``** only.
"""

from __future__ import annotations

from typing import Any

from dejavu_engine import DejavuEngine, render_ir, render_source
from dejavu_language import parse_to_ir
from dejavu_types import DEFAULT_LANGUAGE, normalize, apply_filter, value_to_string

__all__ = [
    "Dejavu",
    "DejavuEngine",
    "DEFAULT_LANGUAGE",
    "parse",
    "render",
    "render_source",
    "normalize",
    "apply_filter",
    "value_to_string",
]


def parse(source: str) -> dict:
    """Parse template source → Dejavu IR document."""
    return parse_to_ir(source)


def render(ir: dict, ctx: dict | None = None) -> str:
    """Render IR + context → string (byte-identical across host languages)."""
    return render_ir(ir, ctx or {})


class Dejavu:
    """Canonical user-facing facade (same role as ``Dejavu`` in other hosts)."""

    @staticmethod
    def parse(source: str) -> dict:
        return parse(source)

    @staticmethod
    def render(ir: dict, ctx: dict | None = None) -> str:
        return render(ir, ctx)

    @staticmethod
    def render_source(source: str, ctx: dict | None = None) -> str:
        return render_source(source, ctx or {})

    @staticmethod
    def check(source: str) -> dict[str, Any]:
        try:
            parse(source)
            return {"valid": True, "errors": []}
        except Exception as e:  # noqa: BLE001
            return {"valid": False, "errors": [str(e)]}
