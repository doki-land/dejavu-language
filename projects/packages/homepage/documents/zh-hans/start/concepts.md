# 核心概念

本页解释渲染 Dejavu 模板时会用到的五个部分。

## 模板

模板由普通输出文本和 Dejavu 指令组成。代码默认写在 `<%` 和 `%>` 之间：

```dejavu
<h1><% page.title %></h1>
```

周围的 HTML 是普通文本；`<% page.title %>` 读取一个值并把它写入结果。

## 上下文

上下文是传给渲染调用的数据：

```ts
const context = {page: {title: "Account settings"}};
```

模板可以读取上下文值，但数据获取和业务决策应留在应用代码中。

## 渲染

渲染把模板和上下文组合起来，返回字符串：

```ts
const html = Dejavu.renderSource("<h1><% page.title %></h1>", context);
```

需要重复渲染时，应用可以只解析一次源码，再使用不同上下文渲染解析结果。

## 转义

插值默认进行 HTML 转义。`<strong>Paid</strong>` 这类值会作为文本输出，不会成为活动标签。只有值已审核或清洗时才能使用 `safe`
或 `raw`；这两个过滤器不会把不可信输入变安全。

## Loader

Loader 给 `extends` 和 `include` 使用的模板分配逻辑名称。应用应按真实相对路径注册一次，例如 `layouts/account.html` 或
`partials/navigation.html`。

单个模板直接传给 `renderSource` 时不需要 loader。模板跨越多个文件时，请阅读[布局与局部模板](../templates/layouts.md)。

接下来了解[插值和缺失值](../templates/interpolation.md)的行为。
