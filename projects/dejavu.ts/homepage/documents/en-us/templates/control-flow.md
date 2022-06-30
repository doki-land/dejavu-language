# Choose and repeat content

Use `if` to select output and `loop` to repeat output for a list. The examples use the default `<%` and `%>` delimiters.

## Choose content

```dejavu
<% if account.suspended %>
  <p>Contact support to restore this account.</p>
<% else if account.trial %>
  <p>Your trial ends on <% account.trialEnd %>.</p>
<% else %>
  <p>Your subscription is active.</p>
<% end if %>
```

`else if` and `else` are optional. Always close the block with `end if`.

## Repeat content

```dejavu
<ul>
<% loop order in orders %>
  <li><% order.number %> - <% order.status %></li>
<% end loop %>
</ul>
```

With two orders, the output is:

```html

<ul>
    <li>A-1042 - paid</li>
    <li>A-1043 - processing</li>
</ul>
```

The current contract supports `loop item in values`. It does not define range loops, tuple destructuring, `break`,
`continue`, or `match`.

## Common errors

- `for` is not the portable loop syntax; use `loop`.
- Use `end if` and `end loop` rather than compact closing keywords from other template languages.
- If a collection may be missing, normalize it to an empty list in application code before rendering.

Continue with [filters](./filters.md).
