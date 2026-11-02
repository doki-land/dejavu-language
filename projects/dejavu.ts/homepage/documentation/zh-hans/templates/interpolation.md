# 插入数据

使用插值把上下文中的值写入渲染结果。

## 读取值

给定以下上下文：

```ts
const context = {
    customer: {
        name: "Mira Chen",
        address: {city: "Suzhou"},
    },
};
```

模板可以读取嵌套成员：

```dejavu
<h1>Welcome, <% customer.name %></h1>
<p>Delivery city: <% customer.address.city %></p>
```

预期输出：

```html
<h1>Welcome, Mira Chen</h1>
<p>Delivery city: Suzhou</p>
```

列表和对象值也可以用方括号访问，例如 `<% orders[0].number %>`。

## 处理缺失值

缺失标识符默认渲染为空字符串。开发期间可以启用严格检查，让缺失名称直接报错：

```ts
Dejavu.renderSource("Account: <% account.name %>", {}, {
    strictUndefined: true,
});
```

只有缺失值属于预期展示状态时才使用默认行为。在测试和构建检查中使用 `strictUndefined`，可以发现拼错的路径。

## 转义

插值默认转义 HTML 字符：

```dejavu
Message: <% message %>
```

传入 `{ message: "<script>alert(1)</script>" }`
时，脚本会作为文本输出。不得为用户控制的值绕过此行为。[过滤器指南](./filters.md)说明了显式的 `safe` 和 `raw` 逃生口。

继续阅读[条件和循环](./control-flow.md)。
