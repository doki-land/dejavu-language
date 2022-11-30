# 渲染第一个模板

本指南用一个模板和 JavaScript 对象渲染订单摘要。

## 开始前

准备一个支持 ES modules 的 TypeScript 或 JavaScript 项目，然后安装公开包：

```bash
pnpm add @doki-land/dejavu
```

不要安装 npm 上无作用域的 `dejavu` 包；它是另一个无关项目。

## 渲染订单摘要

创建 `src/order-summary.ts`：

```ts
import {Dejavu} from "@doki-land/dejavu";

const template = `Order <% order.number %>
<% if order.paid %>Payment received<% else %>Payment required<% end if %>
<% loop item in order.items %>- <% item.name %>: <% item.quantity %>
<% end loop %>`;

const output = Dejavu.renderSource(template, {
    order: {
        number: "A-1042",
        paid: true,
        items: [
            {name: "Notebook", quantity: 2},
            {name: "Pen", quantity: 3},
        ],
    },
});

console.log(output);
```

使用项目现有的 TypeScript 运行器或构建命令执行此文件。

## 预期输出

```text
Order A-1042
Payment received
- Notebook: 2
- Pen: 3
```

## 执行失败时

- 只从 `@doki-land/dejavu` 导入；引擎分层包不是应用 API。
- 插值写成 `<% value %>`，不要照搬其他模板语言的等号形式。
- 条件使用 `end if` 结束，循环使用 `end loop` 结束。
- 名称没有输出时，检查上下文对象中的路径，或按[插值指南](../templates/interpolation.md)启用严格缺失值检查。

继续阅读[核心概念](./concepts.md)，然后[接入模板文件](../integrate/typescript.md)。
