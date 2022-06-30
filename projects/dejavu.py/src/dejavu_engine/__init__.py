from __future__ import annotations

from typing import Any

from dejavu_language import parse_to_ir
from dejavu_types import apply_filter, value_to_string


def html_escape(s: str) -> str:
    return (
        s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&#39;")
    )


def truthy(v: Any) -> bool:
    if v is None:
        return False
    if isinstance(v, bool):
        return v
    if isinstance(v, (int, float)):
        return v != 0
    if isinstance(v, (str, list, dict)):
        return len(v) > 0
    return True


def render_ir(doc: dict, ctx: dict) -> str:
    scope = dict(ctx)
    return _render_node(doc["body"], scope)


def render_source(source: str, ctx: dict) -> str:
    return render_ir(parse_to_ir(source), ctx)


def _render_node(node: dict, scope: dict) -> str:
    t = node["type"]
    if t == "Template":
        return "".join(_render_node(c, scope) for c in node["children"])
    if t == "Text":
        return node["value"]
    if t == "Comment":
        return ""
    if t == "Interpolation":
        s = value_to_string(_eval(node["expression"], scope))
        return s if node.get("raw") else html_escape(s)
    if t == "Stmt.If":
        if truthy(_eval(node["test"], scope)):
            return "".join(_render_node(c, scope) for c in node["consequent"])
        for ei in node.get("elseIfs", []):
            if truthy(_eval(ei["test"], scope)):
                return "".join(_render_node(c, scope) for c in ei["consequent"])
        if "alternate" in node:
            return "".join(_render_node(c, scope) for c in node["alternate"])
        return ""
    if t == "Stmt.For":
        iterable = _eval(node["iterable"], scope)
        if not isinstance(iterable, list):
            raise ValueError("for iterable must be array")
        out = []
        item = node["item"]
        for i, val in enumerate(iterable):
            prev = scope.get(item, _MISSING)
            scope[item] = val
            idx = node.get("index")
            prev_idx = scope.get(idx, _MISSING) if idx else _MISSING
            if idx:
                scope[idx] = i
            out.append("".join(_render_node(c, scope) for c in node["body"]))
            if prev is _MISSING:
                scope.pop(item, None)
            else:
                scope[item] = prev
            if idx:
                if prev_idx is _MISSING:
                    scope.pop(idx, None)
                else:
                    scope[idx] = prev_idx
        return "".join(out)
    if t == "Stmt.Raw":
        return node["value"]
    if t == "Stmt.Block":
        return "".join(_render_node(c, scope) for c in node["body"])
    raise ValueError(f"node not renderable: {t}")


_MISSING = object()


def _eval(expr: dict, scope: dict) -> Any:
    t = expr["type"]
    if t == "Expr.Literal":
        return expr["value"]
    if t == "Expr.Identifier":
        return scope.get(expr["name"])
    if t == "Expr.Member":
        obj = _eval(expr["object"], scope)
        if isinstance(obj, dict):
            return obj.get(expr["property"])
        return None
    if t == "Expr.Index":
        obj = _eval(expr["object"], scope)
        idx = _eval(expr["index"], scope)
        if isinstance(obj, list) and isinstance(idx, int):
            return obj[idx] if 0 <= idx < len(obj) else None
        if isinstance(obj, dict) and isinstance(idx, str):
            return obj.get(idx)
        return None
    if t == "Expr.Binary":
        return _eval_binary(expr["operator"], _eval(expr["left"], scope), _eval(expr["right"], scope))
    if t == "Expr.Unary":
        v = _eval(expr["argument"], scope)
        if expr["operator"] == "!":
            return not truthy(v)
        if expr["operator"] == "-" and isinstance(v, (int, float)):
            return -v
        if expr["operator"] == "+":
            return v
        return None
    if t == "Expr.Pipe":
        val = _eval(expr["expression"], scope)
        args = [_eval(a, scope) for a in expr["arguments"]]
        return apply_filter(expr["filter"], val, args)
    raise ValueError(f"invalid expression: {t}")


def _eval_binary(op: str, l: Any, r: Any) -> Any:
    if op == "+":
        if isinstance(l, (int, float)) and isinstance(r, (int, float)):
            return l + r
        return value_to_string(l) + value_to_string(r)
    if op == "-" and isinstance(l, (int, float)) and isinstance(r, (int, float)):
        return l - r
    if op == "*" and isinstance(l, (int, float)) and isinstance(r, (int, float)):
        return l * r
    if op == "/" and isinstance(l, (int, float)) and isinstance(r, (int, float)):
        return l / r
    if op == "%" and isinstance(l, (int, float)) and isinstance(r, (int, float)):
        return l % r
    if op == "==":
        return l == r
    if op == "!=":
        return l != r
    if op in ("<", "<=", ">", ">=") and isinstance(l, (int, float)) and isinstance(r, (int, float)):
        return {
            "<": l < r,
            "<=": l <= r,
            ">": l > r,
            ">=": l >= r,
        }[op]
    if op == "&&":
        return truthy(l) and truthy(r)
    if op == "||":
        return truthy(l) or truthy(r)
    if op == "in":
        if isinstance(r, list):
            return l in r
        if isinstance(r, str):
            return value_to_string(l) in r
        return False
    return None


class DejavuEngine:
    def parse(self, source: str) -> dict:
        return parse_to_ir(source)

    def render(self, ir: dict, ctx: dict | None = None) -> str:
        return render_ir(ir, ctx or {})

    def render_source(self, source: str, ctx: dict | None = None) -> str:
        return render_source(source, ctx or {})
