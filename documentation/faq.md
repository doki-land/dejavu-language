# 核心开发者 FAQ

## 用户文档在哪里？

外部用户文档位于 `projects/dejavu.ts/homepage/documents/{en-us,zh-hans}/`，站点入口使用 `/d/*` 路径。本目录只写核心实现和贡献流程。

## 公共包和实现包怎么区分？

| 包 | 角色 |
|---|---|
| `@doki-land/dejavu` | **TypeScript 应用唯一入口** |
| `dejavu` (Rust crate) / `use dejavu::*` | **Rust 应用唯一入口** |
| `dejavu` (PyPI) / `Dejavu` (.NET) / `dejavu` (Kotlin) | 其他语言应用入口 |
| `@dejavu/*` | 内部实现，仅绑定/核心维护者 |
| `@doki-land/dejavu-engine` | **Doki 产品**宿主**库**绑定（无 LSP） |
| Rust `dejavu` binary | **唯一** LSP / 完整 IDE 体验（`dejavu lsp`） |

## 规范和实现状态在哪里？

语言和 IR 行为见 [`specifications/`](../specifications/)。各宿主目前通过了哪些测试见[宿主实现状态](./compatibility.md)
。不要把规范中的 MUST 当成所有宿主已经完成的证明。

## Legacy API 还能用吗？

legacy AST 不是正式产品执行面。新增功能必须进入当前 IR 路径，并用 Contract 和 conformance 锁定行为。

## 为什么某些语法没有教程？

当前 Contract 只覆盖已验证的模板核心。未纳入契约的声明、宏、调用、异步 loader 等能力只能写在规划或历史材料中，不能作为可复制的用户语法。
