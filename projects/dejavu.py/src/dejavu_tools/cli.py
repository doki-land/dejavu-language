from __future__ import annotations

import argparse
import json
import sys

from dejavu import Dejavu


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="dejavu")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_parse = sub.add_parser("parse")
    p_parse.add_argument("file")

    p_render = sub.add_parser("render")
    p_render.add_argument("file")
    p_render.add_argument("--from-ir", action="store_true")
    p_render.add_argument("--ctx")

    args = parser.parse_args(argv)
    if args.cmd == "parse":
        ir = Dejavu.parse(open(args.file, encoding="utf-8").read())
        print(json.dumps(ir, ensure_ascii=False, indent=2))
        return 0
    if args.cmd == "render":
        raw = open(args.file, encoding="utf-8").read()
        ctx = json.load(open(args.ctx, encoding="utf-8")) if args.ctx else {}
        out = (
            Dejavu.render(json.loads(raw), ctx)
            if args.from_ir
            else Dejavu.render_source(raw, ctx)
        )
        sys.stdout.write(out)
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
