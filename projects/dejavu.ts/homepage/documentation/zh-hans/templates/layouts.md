# 复用布局与局部模板

多个模板共享页面结构时，配合 loader 使用 `extends`、`block`、`super` 和 `include`。

## 注册模板集合

本例包含三个逻辑文件：

```text
layouts/account.html
pages/profile.html
partials/navigation.html
```

每个路径只注册一次：

```ts
import {Dejavu} from "@doki-land/dejavu";

const engine = Dejavu.withTemplates({
    "layouts/account.html": `
    <header><% block title %>Account<% end block %></header>
    <% include "../partials/navigation.html" %>
    <main><% block content %><% end block %></main>
  `,
    "partials/navigation.html": `<nav>Profile | Security</nav>`,
    "pages/profile.html": `
    <% extends "../layouts/account.html" %>
    <% block title %>Profile - <% super %><% end block %>
    <% block content %><p><% customer.name %></p><% end block %>
  `,
});

const html = engine.renderTemplate("pages/profile.html", {
    customer: {name: "Mira Chen"},
});
```

渲染页面会包含共享布局、导航局部模板、覆盖后的标题和资料内容。

## 引用如何解析

- `../partials/navigation.html` 这样的相对引用从当前模板目录开始解析。
- 裸引用按照已配置 root 的优先级搜索。
- 带 scheme 的引用会明确选择一个已配置 root。
- 路径不能逃逸其 root。

应用应按真实路径注册一次。不要同时用 `profile` 和 `profile.html` 等别名注册同一份源码。

## 常见错误

- `extends` 和 `include` 需要 loader；只有 `renderSource` 无法找到其他模板。
- 相对引用需要具名入口模板，loader 才知道当前目录。
- include 或继承出现循环时会失败，不会无限渲染。
- 子模板继承父模板后，位于 block 外的子模板内容会被丢弃。

继续阅读完整的 [TypeScript 接入指南](../integrate/typescript.md)。
