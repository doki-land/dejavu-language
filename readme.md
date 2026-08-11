# Dejavu

**One template language, shared IR, host-specific runtimes.**

Dejavu is a template engine with a frozen, host-independent contract: parse source into IR, then render with a
JSON-compatible context. TypeScript, Rust, C#, Kotlin, and Python hosts consume the same semantics; application code on
TypeScript uses the public facade `@doki-land/dejavu`.

Repository: [doki-land/dejavu-language](https://github.com/doki-land/dejavu-language/tree/dev) (`dev` branch).

```dejavu
Hello, <% account.name %>!
```

## Features

- Shared **Template Contract** + **IR v1** + conformance fixtures under `specifications/`
- Interpolation, `if` / `else if` / `else`, `loop`, `|>` filters, HTML escaping, `safe` / `raw`
- Inheritance: `extends` / `block` / `super` / `include` with path-v1 loader resolution
- AOT where a host supports it; runtime rendering otherwise
- Host adapters stay outside the public application facade

## Quick start (TypeScript)

```bash
pnpm add @doki-land/dejavu
```

```ts
import {Dejavu, parse, render} from "@doki-land/dejavu";

const out = render(parse("Hello, <% name %>!"), {name: "World"});
// → Hello, World!
```

Install and API notes for other hosts live under each `projects/dejavu.*` tree. Do not depend on `@dejavu/*` or other
engine-layer packages from application code.

## Hosts

| Host       | Public surface      | Notes                      |
|------------|---------------------|----------------------------|
| TypeScript | `@doki-land/dejavu` | Primary application facade |
| Rust       | `dejavu` crate      | Embed / CLI                |
| C#         | `Dejavu`            | .NET binding               |
| Kotlin     | `dejavu`            | JVM / tooling              |
| Python     | `dejavu`            | Binding                    |
| CLI        | `dejavu` binary     | Where packaged             |

Cross-host completeness is recorded in [`documentation/compatibility.md`](./documentation/compatibility.md) — not
implied by directory presence alone.

## Repository layout

| Path                                 | Role                                     |
|--------------------------------------|------------------------------------------|
| `projects/dejavu.ts`                 | TypeScript packages, homepage, user docs |
| `projects/dejavu.rs`                 | Rust crates                              |
| `projects/dejavu.cs` / `.kt` / `.py` | Other host trees                         |
| `specifications/`                    | Contract, IR schema, conformance fixtures |
| `documentation/`                     | Contributor / maintainer docs            |
| `scripts/`                           | Format, test, conformance runners        |

## Documentation

| Kind                | Start here                                                                                                  |
|---------------------|-------------------------------------------------------------------------------------------------------------|
| User docs (en-us)   | [`projects/dejavu.ts/homepage/documents/en-us`](./projects/dejavu.ts/homepage/documents/en-us/index.md)     |
| User docs (zh-hans) | [`projects/dejavu.ts/homepage/documents/zh-hans`](./projects/dejavu.ts/homepage/documents/zh-hans/index.md) |
| Developer docs      | [`documentation/index.md`](./documentation/index.md)                                                        |
| Doc map             | [`documentation/readme.md`](./documentation/readme.md)                                                      |
| Specs               | [`specifications/`](./specifications/)                                                                      |
| Compatibility       | [`documentation/compatibility.md`](./documentation/compatibility.md)                                        |
| Release checklist   | [`documentation/contribute/release.md`](./documentation/contribute/release.md)                              |

## Development

Requires Node.js 20+ and pnpm 10 (see `packageManager` in root `package.json`).

```bash
pnpm install
pnpm fmt:check
pnpm test
pnpm conformance
```

Workspace packages for the TypeScript host live under `projects/dejavu.ts/` (`pnpm-workspace.yaml`).

## Package scope

| Package                                                  | Audience                        |
|----------------------------------------------------------|---------------------------------|
| `@doki-land/dejavu`                                      | Applications                    |
| `@dejavu/engine`, `@dejavu/language`, `@dejavu/types`, … | Binding / core maintainers only |
| Host adapters (`hono-dejavu`, …)                         | Framework integration           |

## License

[MPL-2.0](./License.md)
