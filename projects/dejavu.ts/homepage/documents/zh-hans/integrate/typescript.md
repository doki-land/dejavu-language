# 在 TypeScript 中接入 Dejavu

本指南覆盖应用支持面：直接渲染源码、只解析一次、渲染解析文档、检查源码，以及注册多文件模板。

## 安装公开包

```bash
pnpm add @doki-land/dejavu
```

应用代码只能从 `@doki-land/dejavu` 导入。引擎、语言、IR 和 types 包属于绑定维护者使用的实现分层。

## 直接渲染源码

```ts
import {Dejavu} from "@doki-land/dejavu";

const html = Dejavu.renderSource(
    "<h1><% page.title %></h1>",
    {page: {title: "Billing"}},
    {strictUndefined: true},
);
```

短模板、测试以及不引用其他文件的模板可以使用 `renderSource`。

## 解析一次并重复渲染

```ts
import {parse, render} from "@doki-land/dejavu";

const document = parse("Receipt <% receipt.number %>", {
    file: "receipt.dejavu",
});

const first = render(document, {receipt: {number: "R-1001"}});
const second = render(document, {receipt: {number: "R-1002"}});
```

解析文档是 Dejavu IR 值。普通应用把它作为 `render` 的不透明输入；工具和宿主绑定可以序列化它。

## 检查模板源码

```ts
const result = Dejavu.check("<% if account.active %>Active<% end if %>");

if (!result.valid) {
    console.error(result.errors);
}
```

`check` 校验模板结构，但不会证明运行时上下文一定包含每个属性。

## 注册多文件模板

```ts
const engine = Dejavu.withTemplates({
    "layout.html": `<main><% block body %><% end block %></main>`,
    "dashboard.html": `<% extends "layout.html" %>
    <% block body %><h1><% title %></h1><% end block %>`,
});

const html = engine.renderTemplate("dashboard.html", {
    title: "Operations",
});
```

需要多个 root、优先级覆盖或显式 scheme 时，使用公开包导出的 loader 类。每个具体路径只注册一次，并向 `renderTemplate`
传入规范入口名。

## 错误处理

解析和渲染会报告错误，不会静默改写非法语法。应用日志应保留原始错误消息和诊断详情；向终端用户显示错误时，应删除可能含有秘密的模板源码和上下文值。

继续阅读[布局与局部模板](../templates/layouts.md)、[故障排查](../troubleshoot.md)
和[实现兼容性](../reference/compatibility.md)。
