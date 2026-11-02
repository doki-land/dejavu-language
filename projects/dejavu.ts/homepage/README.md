# DejaVu Homepage

Product homepage and playground for DejaVu, built on the VMZ stack.

## Commands

```bash
pnpm install
pnpm dev       # stage the WASM artifact, then start the dev server
pnpm check     # type and contract checks
pnpm build     # release build into dist/
pnpm preview   # serve the built site
```

## Layout

- `src/Application.vmz` — root shell with the shared header and footer
- `src/pages/` — routes: `index`, `playground`, `ecosystem`, `d`
- `src/components/` — `SiteHeader`, `SiteFooter`
- `src/lib/playground-engine.ts` — playground bridge loader and wire encoding
- `designs/` — brand tokens, theme, and site styles
- `locales/` — `zh-hans` (default) and `en` message catalogs
- `documentation/` — user documentation sources (rendering is wired separately)
- `scripts/sync-wasm.mjs` — stages the bridge module and `.wasm` artifact into `public/vendor/dejavu/`

## Playground

The playground parses and renders templates in the browser through the
`dejavu.wasm/0` bridge published as `@doki-land/dejavu-unknown-wasm32`. The
glue module and `.wasm` binary are staged into `public/` by
`scripts/sync-wasm.mjs`; both are build artifacts and are not committed.
