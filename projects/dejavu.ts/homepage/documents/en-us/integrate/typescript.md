# Integrate Dejavu with TypeScript

This guide covers the supported application-facing API: render source, parse once, render parsed documents, validate
source, and register multi-file templates.

## Install the public package

```bash
pnpm add @doki-land/dejavu
```

Application code imports only from `@doki-land/dejavu`. Engine, language, IR, and types packages are implementation
layers for binding maintainers.

## Render source directly

```ts
import {Dejavu} from "@doki-land/dejavu";

const html = Dejavu.renderSource(
    "<h1><% page.title %></h1>",
    {page: {title: "Billing"}},
    {strictUndefined: true},
);
```

Use `renderSource` for short templates, tests, and templates that do not reference other files.

## Parse once and render repeatedly

```ts
import {parse, render} from "@doki-land/dejavu";

const document = parse("Receipt <% receipt.number %>", {
    file: "receipt.dejavu",
});

const first = render(document, {receipt: {number: "R-1001"}});
const second = render(document, {receipt: {number: "R-1002"}});
```

Parsed documents are Dejavu IR values. Applications normally treat them as opaque inputs to `render`; tools and host
bindings may serialize them.

## Check template source

```ts
const result = Dejavu.check("<% if account.active %>Active<% end if %>");

if (!result.valid) {
    console.error(result.errors);
}
```

`check` validates template structure. It does not validate that every context property will exist at runtime.

## Register multi-file templates

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

For multiple roots, priority overrides, or explicit schemes, use the loader classes exported by the public package.
Register concrete paths once and pass canonical entry names to `renderTemplate`.

## Error handling

Parsing and rendering report errors rather than silently rewriting invalid syntax. Preserve the original error message
and diagnostic details in application logs. When displaying errors to end users, remove template source and context
values that may contain secrets.

See [layouts and partials](../templates/layouts.md), [troubleshooting](../troubleshoot.md),
and [implementation compatibility](../reference/compatibility.md).
