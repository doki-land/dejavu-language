# Dejavu documentation

Use Dejavu to render text or HTML from a template and a data context. The language includes interpolation, conditions,
loops, filters, reusable layouts, and HTML escaping by default.

## Start here

- [Render your first template](./start/quickstart.md) in about five minutes.
- [Learn the core concepts](./start/concepts.md) before building a multi-file template set.
- [Integrate the TypeScript package](./integrate/typescript.md) into an application.

## Write templates

- [Interpolate data](./templates/interpolation.md), access object members, and handle missing values.
- [Choose and repeat content](./templates/control-flow.md) with `if` and `loop`.
- [Transform and escape output](./templates/filters.md) with filters.
- [Reuse layouts and partials](./templates/layouts.md) with `extends`, `block`, and `include`.

## Integrate and maintain

- [TypeScript integration](./integrate/typescript.md)
- [Other host bindings](./integrate/other-hosts.md)
- [Implementation compatibility](./reference/compatibility.md)
- [Troubleshooting](./troubleshoot.md)

The language reference describes portable template behavior. Host bindings may implement different portions of that
behavior; check the compatibility page before choosing one.
