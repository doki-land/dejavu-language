from __future__ import annotations

from typing import Any

DEFAULT_LANGUAGE = {
    "syntaxMode": "template",
    "template": {
        "codeStart": "<%",
        "codeEnd": "%>",
        "commentStart": "<#",
        "commentEnd": "#>",
        "supportFilterPipe": True,
        "legacyFor": False,
    },
}


def normalize(value: Any) -> Any:
    if isinstance(value, list):
        return [normalize(v) for v in value if normalize(v) is not None]
    if isinstance(value, dict):
        if value.get("type") == "Text" and value.get("value") == "":
            return None
        out = {}
        for k in sorted(value.keys()):
            if k == "span":
                continue
            if k == "raw" and value[k] is False:
                continue
            out[k] = normalize(value[k])
        return out
    return value


def value_to_string(v: Any) -> str:
    if v is None:
        return ""
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, (int, float, str)):
        return str(v)
    return str(v)


def apply_filter(name: str, value: Any, args: list[Any]) -> Any:
    if name == "uppercase":
        return value_to_string(value).upper()
    if name == "lowercase":
        return value_to_string(value).lower()
    if name == "trim":
        return value_to_string(value).strip()
    if name == "default":
        if value is None or value == "":
            return args[0] if args else None
        return value
    if name == "length":
        if isinstance(value, (str, list, dict)):
            return len(value)
        return 0
    if name == "join":
        sep = value_to_string(args[0]) if args else ","
        if isinstance(value, list):
            return sep.join(value_to_string(x) for x in value)
        return value_to_string(value)
    if name == "replace":
        frm = value_to_string(args[0]) if args else ""
        to = value_to_string(args[1]) if len(args) > 1 else ""
        return value_to_string(value).replace(frm, to)
    raise ValueError(f"unknown filter `{name}`")
