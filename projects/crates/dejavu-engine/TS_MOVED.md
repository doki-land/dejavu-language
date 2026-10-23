# TypeScript engine moved

The TypeScript implementation lives only under `projects/packages/` and is a **native TypeScript stack** (no
WASM/WASI bridge).

| Package                     | Role                                            |
|-----------------------------|-------------------------------------------------|
| `@doki-land/dejavu-engine`  | Pure TypeScript AOT runtime used by generated modules |
| `@doki-land/dejavu`         | N-API facade, JIT API, and `dejavu` CLI         |

Cross-language contract: repo-root `specifications/ir` + `../../../specifications/conformance/t1`.

Legacy hand-written lexer/parser/renderer copies (migration reference only) may remain under:

`../../packages/dejavu-engine/src/legacy/`
