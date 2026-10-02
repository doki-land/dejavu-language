# Troubleshoot Dejavu templates

Start with the visible symptom, then reduce the template and context to the smallest failing example.

## A value renders as an empty string

A missing identifier becomes an empty string by default. Check spelling and nesting in the context object. Enable
`strictUndefined: true` during development to turn missing names into errors.

## HTML appears as `&lt;` and `&gt;`

Interpolation escapes HTML by default. This is expected for ordinary and user-controlled text. Use `safe` or `raw` only
for HTML that the application has already sanitized or generated from a trusted source.

## A filter reports an error

Use `|>` rather than a single `|`. Verify the built-in name and its arguments. The supported list is in
the [filters guide](./templates/filters.md).

## `if` or `loop` does not parse

Use the portable closing forms:

```dejavu
<% if ready %>Ready<% end if %>
<% loop item in items %><% item %><% end loop %>
```

Do not copy equals-sign interpolation or compact closing keywords from other template languages.

## `extends` or `include` cannot find a template

- Confirm that the application configured a loader.
- Register the template under its real relative path exactly once.
- For `./` or `../` references, render from a named entry so the loader knows the current template.
- Check root priority or use an explicit scheme when the same path exists in multiple roots.
- A relative path cannot escape its configured root.

## The package import resolves to the wrong library

Install and import `@doki-land/dejavu`. The unscoped npm package named `dejavu` is a different project.

## Results differ between hosts

First render the same IR and JSON context on both hosts. Then check
the [compatibility page](./reference/compatibility.md). A specification requirement does not prove that every
implementation has completed that requirement.

## Report a reproducible issue

Include the smallest template, a redacted JSON context, host language and package version, exact error or diagnostic
code, and expected versus actual output. Do not attach secrets or production customer data.
