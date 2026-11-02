# Transform and escape output

Filters transform a value before it is written. Connect filters with `|>`; a single `|` is not valid Dejavu syntax.

## Transform values

```dejavu
<h1><% article.title |> trim |> upper %></h1>
<p>Tags: <% article.tags |> join(", ") %></p>
```

Filters run from left to right. Arguments can use call form, such as `join(", ")`, or the contract's colon form.

The current built-ins are:

| Purpose                   | Filters                                                                 |
|---------------------------|-------------------------------------------------------------------------|
| Text                      | `upper` / `uppercase`, `lower` / `lowercase`, `trim`, `replace`, `slug` |
| Collections and fallbacks | `length`, `join`, `default`                                             |
| Formatting                | `date`                                                                  |
| HTML handling             | `escape` / `e`, `safe` / `raw`                                          |

## Render trusted HTML

Interpolation escapes HTML by default. `safe` and `raw` disable that escaping:

```dejavu
<article><% article.reviewedHtml |> safe %></article>
```

Use these filters only for content that the application has already sanitized or generated from trusted sources. `safe`
and `raw` do not inspect, clean, or validate HTML.

To force escaping even when a host value was marked safe, use `escape`:

```dejavu
<code><% article.reviewedHtml |> escape %></code>
```

## Common errors

- Use `|>` rather than a single `|`.
- Check the filter name and argument count when rendering reports an unknown filter.
- Do not use `safe` to hide unexpected escaping. First determine whether the value should be HTML at all.

Continue with [layouts and partials](./layouts.md).
