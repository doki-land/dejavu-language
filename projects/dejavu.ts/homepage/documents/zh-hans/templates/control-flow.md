# 选择和重复内容

使用 `if` 选择输出，使用 `loop` 为列表重复输出。以下示例使用默认定界符 `<%` 和 `%>`。

## 选择内容

```dejavu
<% if account.suspended %>
  <p>Contact support to restore this account.</p>
<% else if account.trial %>
  <p>Your trial ends on <% account.trialEnd %>.</p>
<% else %>
  <p>Your subscription is active.</p>
<% end if %>
```

`else if` 和 `else` 可以省略。代码块始终使用 `end if` 结束。

## 重复内容

```dejavu
<ul>
<% loop order in orders %>
  <li><% order.number %> - <% order.status %></li>
<% end loop %>
</ul>
```

传入两条订单时，输出为：

```html

<ul>
    <li>A-1042 - paid</li>
    <li>A-1043 - processing</li>
</ul>
```

当前契约支持 `loop item in values`，不定义范围循环、元组解构、`break`、`continue` 或 `match`。

## 常见错误

- `for` 不是可移植循环语法，应使用 `loop`。
- 使用 `end if` 和 `end loop`，不要使用其他模板语言的紧凑结束关键字。
- 集合可能缺失时，在应用代码中先把它规范化为空列表。

继续阅读[过滤器](./filters.md)。
