# 🚀 dejavu (Rust)

**Public Rust surface for the Dejavu template language.**

Application code should depend on the **`dejavu`** crate only and import via `dejavu::*` (for example `use dejavu::Dejavu`). Internal crates (`dejavu-language`, `dejavu-ir`, `dejavu-engine`, `dejavu-runtime`, `dejavu-types`) are implementation details.

## 📦 Installation

```toml
[dependencies]
dejavu = "0.0.1"
serde_json = "1"
```

## 🚀 Default usage (`<% %>` delimiters)

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

## 🔧 Non-default delimiters (Doki / Jinja / …)

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

## 📨 What the crate gives you

| API                                  | Purpose                                        |
|--------------------------------------|------------------------------------------------|
| `Dejavu::parse(source)`              | Parse source → IR (default delimiters)         |
| `Dejavu::parse_with(source, &lang)`  | Parse source → IR (custom delimiters)          |
| `Dejavu::render(doc, &ctx)`          | Render an IR document                          |
| `Dejavu::render_source(source, &ctx)` | Parse + render in one call (default)           |
| `Dejavu::render_source_with(…)`      | Parse + render with custom delimiters          |
| `Dejavu::check(source)` / `check_with(…)` | Syntax check only                           |

Also re-exported: `DejavuEngine` (stateful facade), IR types (`IrDocument`, `IrNode`, …), runtime helpers (`Template`, `Escaper`, `EscapeDisplay`, …), and the diagnostic contract (`Diagnostic`, `Severity`, `Span`).

## 📄 License

[MPL-2.0](../../License.md)
