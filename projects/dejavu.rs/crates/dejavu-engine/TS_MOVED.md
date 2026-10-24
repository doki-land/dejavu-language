# TypeScript packages layout

TypeScript 实现位于 `projects/dejavu.ts/`：

| 路径 | 包 | 角色 |
|------|-----|------|
| `packages/dejavu` | `@doki-land/dejavu` | 应用 facade、CLI、`dejavu generate` |
| `packages/dejavu-engine` | `@doki-land/dejavu-engine` | AOT 生成模块的运行时（`runtime` 子路径） |
| `platforms/dejavu-*` | `@doki-land/dejavu-*` | 各平台 N-API 原生 addon |

pnpm workspace 只包含 `packages/dejavu`、`packages/dejavu-engine`、`packages/dejavu-skills` 与 `platforms/dejavu-*`。

跨语言契约（自任意 `projects/dejavu.rs/crates/<crate>/` 测试目录）：

- IR 与 schema：`../../../../specifications/ir`
- Conformance 金样：`../../../../specifications/conformance/t1`

`packages/dejavu-language/` 保留 TypeScript 参考 parser，不在 workspace 内；正式 JIT / codegen 上游为 Rust `dejavu-language` + `dejavu-codegen`。
