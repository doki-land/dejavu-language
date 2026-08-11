# 🚀 dejavu (Rust)

**Public Rust surface for the Dejavu template language.**

Application code should depend on the **`dejavu`** crate only and import via `dejavu::*`
(for example `use dejavu::Dejavu`). Internal crates (`dejavu-language`, `dejavu-ir`,
`dejavu-engine`, `dejavu-runtime`, `dejavu-types`) are implementation details.

## 📦 Installation

```toml
[dependencies]
dejavu = "0.0.1"
```

## 🚀 Usage

```rust
use dejavu::Dejavu;
use serde_json::json;

fn main() {
    let output = Dejavu::render_source(
        "Hello, <% account.name %>!",
        &json!({ "account": { "name": "Mira" } }),
    )
    .expect("render");
    println!("{output}");
}
```

Also re-exported: `parse`, `render`, `render_source`, `DejavuEngine`, IR types, and runtime helpers
(`Template`, `Escaper`, …) for advanced embedding — prefer `Dejavu::*` in application tutorials.

## 📄 License

MPL-2.0
