# ✨ Dejavu Language

**One template language, shared IR, host-specific runtimes.**

Dejavu separates template intent from the host runtime. Write a template once, then pick the binding that fits your app. A shared IR keeps behavior aligned across TypeScript, Rust, C#, Kotlin, and Python.

Repository: [`doki-land/dejavu-language`](https://github.com/doki-land/dejavu-language/tree/dev) (`dev`).

```dejavu
Hello, <% account.name %>!
```

## ✨ What you get

- One template language surface (Template Contract + IR v1)
- Shared conformance fixtures under `specifications/`
- Interpolation, `if` / `loop`, `|>` filters, HTML escaping, `safe` / `raw`
- Inheritance: `extends` / `block` / `super` / `include` + path-v1 loader
- Configurable delimiters — default `<% %>`, Doki uses `{% %}`, Jinja anything
- AOT where a host supports it; runtime rendering otherwise

## 🚀 Quick start

### TypeScript

```bash
pnpm add @doki-land/dejavu
```

```ts
import { render } from "@doki-land/dejavu";

const out = render("Hello, <% name %>!", { name: "World" });
// → Hello, World!
```

### Rust

```toml
[dependencies]
dejavu = "0.0.1"
serde_json = "1"
```

```rust
use dejavu::Dejavu;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = Dejavu::render_source(
        "Hello, <% account.name %>!",
        &json!({ "account": { "name": "Mira" } }),
    )?;
    println!("{out}"); // → Hello, Mira!
    Ok(())
}
```

### Non-default delimiters (Doki / Jinja)

```rust
use dejavu::{Dejavu, Language, TemplateConfig};

let doki = Language {
    syntax_mode: "template".into(),
    template: TemplateConfig {
        code_start: "{%".into(),
        code_end: "%}".into(),
        comment_start: "{#".into(),
        comment_end: "#}".into(),
        support_filter_pipe: true,
        legacy_for: false,
    },
};

let out = Dejavu::render_source_with(
    "Hello, {% name %}!",
    &json!({ "name": "Mira" }),
    &doki,
)?;
```

## 🧩 Hosts

| Host       | Public surface                         | Status   | Notes                                      |
|------------|----------------------------------------|----------|--------------------------------------------|
| TypeScript | `@doki-land/dejavu`                    | ✅ Active | Application facade + CLI over the N-API engine |
| Rust       | `dejavu` (`use dejavu::*`)             | ✅ Active | Application facade + public crate workspace |
| C#         | `Dejavu`                               | 🔒 Frozen | .NET binding — protocol consumer only      |
| Kotlin     | `dejavu`                               | 🔒 Frozen | JVM / tooling — protocol consumer only     |
| Python     | `dejavu`                               | 🔒 Frozen | Binding                                     |
| CLI        | `doki` (entry) + `dejavu-tools` (rust) | ✅ Active | Doki product CLI + standalone lint/extract |
| LSP        | `dejavu-tools` (`dejavu lsp`)          | 🧪 Draft | Rust-only — not shipped to npm             |
| Doki core  | `doki-core` (this repo)                | ✅ Active | SSG build kernel — consumes the `dejavu` crate |

Cross-host status matrix: [`documentation/compatibility.md`](./documentation/compatibility.md).

## 🗂 Repository layout

| Path                                  | Role                                      |
|---------------------------------------|-------------------------------------------|
| `projects/dejavu.rs/crates/`          | Rust crates (`dejavu`, `dejavu-language`, `dejavu-ir`, `dejavu-codegen`, …) |
| `projects/dejavu.ts/packages/`        | Published TS packages (`dejavu`, `dejavu-engine`, …) |
| `projects/dejavu.ts/platforms/`       | Per-platform N-API native addons          |
| `projects/dejavu.ts/homepage/`        | User docs site sources                    |
| `projects/dejavu.cs` / `.kt` / `.py`  | Other host trees (frozen protocol consumers) |
| `specifications/`                     | Contract, IR schema, conformance fixtures |
| `documentation/`                      | Contributor / maintainer docs             |
| `scripts/`                            | Format, test, conformance runners         |

## 📚 Documentation

| Kind                | Start here                                                                                                  |
|---------------------|-------------------------------------------------------------------------------------------------------------|
| User docs (en-us)   | [`projects/dejavu.ts`](projects/dejavu.ts/homepage/documents/en-us/index.md)     |
| User docs (zh-hans) | [`projects/dejavu.ts`](projects/dejavu.ts/homepage/documents/zh-hans/index.md) |
| Developer docs      | [`documentation/index.md`](./documentation/index.md)                                                        |
| Doc map             | [`documentation/readme.md`](./documentation/readme.md)                                                      |
| Specs               | [`specifications/`](./specifications/)                                                                      |
| Conformance         | [`scripts/conformance-runner.mjs`](./scripts/conformance-runner.mjs)                                        |
| Compatibility       | [`documentation/compatibility.md`](./documentation/compatibility.md)                                        |
| Release checklist   | [`documentation/contribute/release.md`](./documentation/contribute/release.md)                              |

## 🛠 Development

Requires Node.js 20+ and pnpm 10 (`packageManager` in root `package.json`), plus a Rust toolchain
for the N-API engine addon.

```bash
# From repo root
pnpm install
pnpm napi:build:release      # builds + stages the native addon
pnpm fmt:check
pnpm test                    # Rust workspace + @doki-land/dejavu tests
pnpm conformance             # cross-host conformance runner (requires hosts)
```

## 📦 Package scope

| Package / Crate                                          | Audience                          |
|----------------------------------------------------------|-----------------------------------|
| `@doki-land/dejavu`                                      | TypeScript applications (JIT + CLI + `dejavu generate`) |
| `@doki-land/dejavu-engine`                               | Generated AOT modules (`/runtime` import only) |
| `dejavu` (crates.io)                                     | Rust applications (`use dejavu::*`) |
| `dejavu-language` / `dejavu-ir` / `dejavu-codegen`       | Binding / core maintainers only   |
| `dejavu-napi`                                            | N-API binding only                |

## 📄 License

[MPL-2.0](./License.md)
