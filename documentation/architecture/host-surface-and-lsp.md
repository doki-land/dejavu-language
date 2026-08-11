# 宿主表面与 LSP 策略

## 应用入口（库）

| 宿主 | 包 / crate | 用途 |
|------|------------|------|
| TypeScript | `@doki-land/dejavu` | 库：parse / render / loader |
| Rust | `dejavu`（`use dejavu::*`） | 库：同左 |
| Python / C# / Kotlin | `dejavu` / `Dejavu` | 库：同左 |
| Doki 产品绑定 | `@doki-land/dejavu-engine` | Doki 定界符与 SSG 适配（仍是库） |

`@dejavu/*`、各语言内部 crate（`dejavu-engine`、`dejavu-ir` …）不是应用入口。

## IDE / LSP（唯一）

**只有 Rust `dejavu` 二进制提供 Language Server。**

```bash
dejavu lsp
```

- TypeScript / Python / C# / Kotlin 宿主 **不** 发布 LSP。
- 编辑器扩展应 spawn `dejavu`（stdio），而不是内嵌 Node/JVM language server。
- 需要「完整语言体验」（诊断、补全、跳转等）时：安装本仓库发布的 Rust 产物（GitHub Release zip / GHCR 镜像 / 本地 `cargo install`），与所用宿主库版本无关。

## 禁止

- 在 `@doki-land/dejavu`、`@doki-land/dejavu-engine` 或其他语言库包中新增 LSP。
- 把 Node `@doki-land/doki-lsp` 当作正式面（已废弃，仅历史参考）。
