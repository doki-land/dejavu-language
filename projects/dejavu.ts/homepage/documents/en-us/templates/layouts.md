# Reuse layouts and partials

Use a loader with `extends`, `block`, `super`, and `include` when several templates share page structure.

## Register the template set

This example has three logical files:

```text
layouts/account.html
pages/profile.html
partials/navigation.html
```

Register each path once:

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

The rendered page contains the shared layout, navigation partial, overridden title, and profile content.

## How references resolve

- Relative references such as `../partials/navigation.html` resolve from the current template's directory.
- Bare references search configured roots in priority order.
- Scheme-qualified references select one configured root explicitly.
- A path cannot escape its root.

Applications should register the real path once. Do not register duplicate aliases such as both `profile` and
`profile.html` for the same source.

## Common errors

- `extends` and `include` require a loader; `renderSource` alone cannot find other templates.
- Relative references need a named entry template so the loader knows the current directory.
- Include cycles and inheritance cycles fail instead of rendering indefinitely.
- Content outside child blocks is discarded when a template extends a parent.

Continue with the complete [TypeScript integration guide](../integrate/typescript.md).
