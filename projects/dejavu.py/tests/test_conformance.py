from __future__ import annotations

import json
from pathlib import Path

import pytest

from dejavu import Dejavu
from dejavu_engine import DejavuEngine
from dejavu_language import ParseError
from dejavu_types import normalize

ROOT = Path(__file__).resolve().parents[3] / "specifications" / "conformance" / "t1"


def test_t1_cases():
    assert ROOT.is_dir(), ROOT
    engine = DejavuEngine()
    for case_dir in sorted(p for p in ROOT.iterdir() if p.is_dir()):
        input_src = (case_dir / "input.dejavu").read_text(encoding="utf-8")
        expected_ir = json.loads((case_dir / "expected.ir.json").read_text(encoding="utf-8"))
        ctx = json.loads((case_dir / "context.ctx.json").read_text(encoding="utf-8"))
        expected_out = (case_dir / "expected.out.txt").read_text(encoding="utf-8")

        got = engine.parse(input_src)
        assert normalize(got) == normalize(expected_ir), case_dir.name
        assert engine.render(expected_ir, ctx) == expected_out, case_dir.name


def test_control_statements_inside_loop_lower_to_nodes():
    engine = DejavuEngine()
    break_ir = engine.parse("<% loop x in xs %><% break %><% end loop %>")
    body = break_ir["body"]["children"][0]["body"]
    assert body[0] == {"type": "Stmt.Break", "trim": "none"}

    continue_ir = engine.parse("<% loop x in xs %><% continue %><% end loop %>")
    body = continue_ir["body"]["children"][0]["body"]
    assert body[0] == {"type": "Stmt.Continue", "trim": "none"}


def test_control_statements_render_from_source():
    assert (
        Dejavu.render_source(
            "<% loop x in xs %><% if x == 2 %><% break %><% end if %><% x %><% end loop %>",
            {"xs": [1, 2, 3]},
        )
        == "1"
    )
    assert (
        Dejavu.render_source(
            "<% loop x in xs %><% if x == 2 %><% continue %><% end if %><% x %><% end loop %>",
            {"xs": [1, 2, 3]},
        )
        == "13"
    )


def test_break_in_nested_loop_binds_the_inner_loop():
    assert (
        Dejavu.render_source(
            "<% loop a in xs %><% loop b in ys %><% break %><% end loop %><% a %><% end loop %>",
            {"xs": [1, 2], "ys": [9]},
        )
        == "12"
    )


@pytest.mark.parametrize(
    ("source", "kind"),
    [
        ("<% break %>", "break"),
        ("<% continue %>", "continue"),
        ("<% if true %><% break %><% end if %>", "break"),
    ],
)
def test_control_statement_outside_loop_is_a_parse_error(source: str, kind: str):
    with pytest.raises(ParseError) as err:
        Dejavu.parse(source)
    assert err.value.code == "dejavu.parse.control-outside-loop"
    assert err.value.diagnostic_args == {"kind": kind}
