# TypeScript engine moved

The TypeScript implementation lives only under `projects/dejavu.ts/packages/` and is a **native TypeScript stack** (no
WASM/WASI bridge).

| Package            | Role                          |
|--------------------|-------------------------------|
| `@dejavu/language` | `source → IR`                 |
| `@dejavu/types`    | IR types, normalize, filters  |
| `@dejavu/engine`   | `render(IR)` / `renderSource` |
| `@dejavu/tools`    | CLI around the above          |

Cross-language contract: repo-root `specifications/ir` + `../../../specifications/conformance/t1`.

Legacy hand-written lexer/parser/renderer copies (migration reference only) may remain under:

`../../dejavu.ts/dejavu-engine/src/legacy/`
