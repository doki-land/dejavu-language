"""Run T1 conformance without pytest."""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from dejavu import Dejavu, normalize  # noqa: E402

T1 = ROOT / "specifications" / "conformance" / "t1"


def main() -> int:
    failed = 0
    for case_dir in sorted(p for p in T1.iterdir() if p.is_dir()):
        input_src = (case_dir / "input.dejavu").read_text(encoding="utf-8")
        expected_ir = json.loads((case_dir / "expected.ir.json").read_text(encoding="utf-8"))
        ctx = json.loads((case_dir / "context.ctx.json").read_text(encoding="utf-8"))
        expected_out = (case_dir / "expected.out.txt").read_text(encoding="utf-8")
        got = Dejavu.parse(input_src)
        if normalize(got) != normalize(expected_ir):
            print(f"FAIL IR {case_dir.name}")
            failed += 1
            continue
        out = Dejavu.render(expected_ir, ctx)
        if out != expected_out:
            print(f"FAIL OUT {case_dir.name}: {out!r} != {expected_out!r}")
            failed += 1
            continue
        print(f"OK {case_dir.name}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
