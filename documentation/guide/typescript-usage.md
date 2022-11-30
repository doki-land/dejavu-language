# TypeScript 公开表面

本页供 TypeScript facade 维护者核对公开 API。用户教程见 `/d/zh-hans/integrate/typescript`。

应用安装并导入 `@doki-land/dejavu`：

```ts
import { Dejavu, parse, render } from "@doki-land/dejavu";

const document = parse("Hello, <% account.name %>!");
const output = render(document, { account: { name: "Mira" } });
const direct = Dejavu.renderSource("Hello, <% account.name %>!", {
  account: { name: "Mira" },
});
```

公开 facade 还负责 `check`、`withTemplates` 和公开 loader 类型。`@doki-land/dejavu-engine`、language、types 等实现包不得成为普通应用教程的依赖入口。

维护 facade 时必须验证：

- 具名导出与 `Dejavu` facade 的语义一致；
- options 不泄漏宿主私有 IR 类型；
- package exports 指向发布产物；
- quickstart 在干净项目中可运行。
