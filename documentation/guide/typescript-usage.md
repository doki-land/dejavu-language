# TypeScript 公开表面

本页供 TypeScript facade 维护者核对公开 API。用户教程见 `/d/zh-hans/integrate/typescript`。

应用安装并导入 `@doki-land/dejavu`：

```ts
import { render, compile } from "@doki-land/dejavu";

const output = render("Hello, <% account.name %>!", { account: { name: "Mira" } });

const template = compile("Hello, <% account.name %>!");
const again = template.render({ account: { name: "Mira" } });
```

公开 facade 还负责 `parse`、`precompile`、`precompileCatalog`、`renderIr`、`resolveTemplateRef` 与 `DejavuError`。

- 普通应用：`@doki-land/dejavu`（JIT API + `dejavu` CLI + `dejavu generate`）
- 生成代码的运行时：`@doki-land/dejavu-engine`（纯 TypeScript AOT 运行库，只被生成模块 import）
- 生成代码：`src/generated/dejavu/` 下的 ESM 模块，固定 import `@doki-land/dejavu-engine/runtime`

维护 facade 时必须验证：

- 具名导出与发布产物一致；
- options 不泄漏宿主私有 IR 类型；
- package exports 指向发布产物，且 `files` 覆盖运行时需要的 `locales/`；
- quickstart 与 `dejavu generate` 在干净项目中可运行。
