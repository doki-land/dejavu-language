# Core concepts

This page explains the five pieces you use when rendering Dejavu templates.

## Template

A template contains ordinary output text and Dejavu instructions. Code uses `<%` and `%>` by default:

```dejavu
<h1><% page.title %></h1>
```

The surrounding HTML is ordinary text. `<% page.title %>` reads a value and writes it to the result.

## Context

The context is the data passed to a render call:

```ts
const context = {page: {title: "Account settings"}};
```

Templates can read context values but should leave data fetching and business decisions in application code.

## Render

Rendering combines a template with a context and returns a string:

```ts
const html = Dejavu.renderSource("<h1><% page.title %></h1>", context);
```

For repeated rendering, an application may parse source once and render the parsed document with different contexts.

## Escaping

Interpolated values are HTML-escaped by default. A value such as `<strong>Paid</strong>` is emitted as text, not active
markup. Use `safe` or `raw` only when the value has already been reviewed or sanitized; these filters do not make
untrusted input safe.

## Loader

A loader gives logical names to templates used by `extends` and `include`. Applications register each template once
under its real relative path, such as `layouts/account.html` or `partials/navigation.html`.

You do not need a loader for a single template passed to `renderSource`.
See [layouts and partials](../templates/layouts.md) when your templates span files.

Next, learn how [interpolation and missing values](../templates/interpolation.md) behave.
