from __future__ import annotations

import json
from pathlib import Path

from dejavu_engine import DejavuEngine
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
