# ✨ Dejavu Language

**One template language, shared IR, host-specific runtimes.**

Dejavu separates template intent from the host runtime. Write a template once, then pick the binding that fits your app.
A shared IR keeps behavior aligned across TypeScript, Rust, C#, Kotlin, and Python.

Repository: [`doki-land/dejavu-language`](https://github.com/doki-land/dejavu-language/tree/dev) (`dev`).

```dejavu
Hello, <% account.name %>!
```

## ✨ What you get

- One template language surface (Template Contract + IR v1)
- Shared conformance fixtures under `specifications/`
- Interpolation, `if` / `loop`, `|>` filters, HTML escaping, `safe` / `raw`
- Inheritance: `extends` / `block` / `super` / `include` + path-v1 loader
- AOT where a host supports it; runtime rendering otherwise
- Host adapters kept separate from the public `@doki-land/dejavu` facade

## 🚀 Quick start (TypeScript)

```bash
pnpm add @doki-land/dejavu
```

```ts
import {Dejavu, parse, render} from "@doki-land/dejavu";

const out = render(parse("Hello, <% name %>!"), {name: "World"});
// → Hello, World!
```

Install and API notes for other hosts live under each `projects/dejavu.*` tree. Application code should not depend on
`@dejavu/*` engine-layer packages.

## 🧩 Hosts

| Host       | Public surface                         | Notes                                      |
|------------|----------------------------------------|--------------------------------------------|
| TypeScript | `@doki-land/dejavu`                    | Application facade                         |
| Rust       | `dejavu` (`use dejavu::*`)             | Application facade                         |
| C#         | `Dejavu`                               | .NET binding                               |
| Kotlin     | `dejavu`                               | JVM / tooling                              |
| Python     | `dejavu`                               | Binding                                    |
| CLI        | `dejavu` binary                        | From `dejavu-tools`                        |
| Doki only  | `@doki-land/dejavu-engine`             | Product host adapter — not general apps    |

Cross-host status: [`documentation/compatibility.md`](./documentation/compatibility.md).

## 🗂 Repository layout

| Path                                 | Role                                      |
|--------------------------------------|-------------------------------------------|
| `projects/dejavu.ts`                 | TypeScript packages, homepage, user docs  |
| `projects/dejavu.rs`                 | Rust crates                               |
| `projects/dejavu.cs` / `.kt` / `.py` | Other host trees                          |
| `specifications/`                    | Contract, IR schema, conformance fixtures |
| `documentation/`                     | Contributor / maintainer docs             |
| `scripts/`                           | Format, test, conformance runners         |

## 📚 Documentation

| Kind                | Start here                                                                                                  |
|---------------------|-------------------------------------------------------------------------------------------------------------|
| User docs (en-us)   | [`projects/dejavu.ts/homepage/documents/en-us`](./projects/dejavu.ts/homepage/documents/en-us/index.md)     |
| User docs (zh-hans) | [`projects/dejavu.ts/homepage/documents/zh-hans`](./projects/dejavu.ts/homepage/documents/zh-hans/index.md) |
| Developer docs      | [`documentation/index.md`](./documentation/index.md)                                                        |
| Doc map             | [`documentation/readme.md`](./documentation/readme.md)                                                      |
| Specs               | [`specifications/`](./specifications/)                                                                      |
| Compatibility       | [`documentation/compatibility.md`](./documentation/compatibility.md)                                        |
| Release checklist   | [`documentation/contribute/release.md`](./documentation/contribute/release.md)                              |

## 🛠 Development

Requires Node.js 20+ and pnpm 10 (`packageManager` in root `package.json`).

```bash
pnpm install
pnpm fmt:check
pnpm test
pnpm conformance
```

## 📦 Package scope

| Package                                                  | Audience                                   |
|----------------------------------------------------------|--------------------------------------------|
| `@doki-land/dejavu`                                      | TypeScript applications                    |
| `dejavu` (crates.io)                                     | Rust applications (`use dejavu::*`)        |
| `@dejavu/engine`, `@dejavu/language`, `@dejavu/types`, … | Binding / core maintainers only            |
| `@doki-land/dejavu-engine`                               | Doki product host adapter only             |
| Host adapters (`hono-dejavu`, …)                         | Framework integration                      |

## 📄 License

[MPL-2.0](./License.md)
