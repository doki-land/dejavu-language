# Render your first template

This guide renders an order summary from a template and a JavaScript object.

## Before you start

Use a TypeScript or JavaScript project that supports ES modules. Install the public package:

```bash
pnpm add @doki-land/dejavu
```

Do not install the unrelated unscoped `dejavu` package from npm.

## Render an order summary

Create `src/order-summary.ts`:

```ts
import {Dejavu} from "@doki-land/dejavu";

const template = `Order <% order.number %>
<% if order.paid %>Payment received<% else %>Payment required<% end if %>
<% loop item in order.items %>- <% item.name %>: <% item.quantity %>
<% end loop %>`;

const output = Dejavu.renderSource(template, {
    order: {
        number: "A-1042",
        paid: true,
        items: [
            {name: "Notebook", quantity: 2},
            {name: "Pen", quantity: 3},
        ],
    },
});

console.log(output);
```

Run the file with the TypeScript runner or build command used by your project.

## Expected output

```text
Order A-1042
Payment received
- Notebook: 2
- Pen: 3
```

## If it fails

- Import only from `@doki-land/dejavu`; engine-layer packages are not the application API.
- Interpolation uses `<% value %>`, not the equals-sign form used by some template languages.
- Close conditions with `end if` and loops with `end loop`.
- If a name produces no output, check its path in the context object or enable strict missing-value checks in
  the [interpolation guide](../templates/interpolation.md).

Continue with [core concepts](./concepts.md), then [integrate template files](../integrate/typescript.md).
