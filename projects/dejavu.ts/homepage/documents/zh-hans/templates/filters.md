# 转换并转义输出

过滤器在值写入结果前对它进行转换。过滤器使用 `|>` 连接；单独的 `|` 不是合法 Dejavu 语法。

## 转换值

```dejavu
<h1><% article.title |> trim |> upper %></h1>
<p>Tags: <% article.tags |> join(", ") %></p>
```

过滤器从左向右执行。参数可以使用 `join(", ")` 这样的调用形式，也可以使用契约定义的冒号形式。

当前内置过滤器包括：

| 用途         | 过滤器                                                                  |
|--------------|-------------------------------------------------------------------------|
| 文本         | `upper` / `uppercase`、`lower` / `lowercase`、`trim`、`replace`、`slug` |
| 集合与默认值 | `length`、`join`、`default`                                             |
| 格式化       | `date`                                                                  |
| HTML 处理    | `escape` / `e`、`safe` / `raw`                                          |

## 渲染可信 HTML

插值默认转义 HTML。`safe` 和 `raw` 会关闭转义：

```dejavu
<article><% article.reviewedHtml |> safe %></article>
```

只有应用已经清洗，或由可信来源生成的内容才能使用这两个过滤器。`safe` 和 `raw` 不会检查、清理或验证 HTML。

即使宿主值已被标记为安全，也可以用 `escape` 强制转义：

```dejavu
<code><% article.reviewedHtml |> escape %></code>
```

## 常见错误

- 使用 `|>`，不要使用单独的 `|`。
- 渲染报告未知过滤器时，检查名称和参数数量。
- 不要用 `safe` 掩盖意外转义；先确认这个值是否本来就应该作为 HTML 输出。

继续阅读[布局与局部模板](./layouts.md)。
