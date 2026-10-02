# 🚀 DejaVu Macros

[![Crates.io](https://img.shields.io/crates/v/dejavu-macros.svg)](https://crates.io/crates/dejavu-macros)
[![Documentation](https://docs.rs/dejavu-macros/badge.svg)](https://docs.rs/dejavu-macros)

**Powerful Macros for Template Engines.** `dejavu-macros` provides procedural macros for the DejaVu template engine,
enabling compile-time template processing and seamless integration with Rust code.

## ✨ Core Features

- **🔧 Procedural Macros**：Powerful compile-time template processing
- **⚡ Zero Runtime Overhead**：Templates compiled directly to Rust code
- **🎯 Type Safety**：Compile-time validation of template syntax
- **🛠️ Seamless Integration**：Templates as part of Rust codebase
- **📦 No External Dependencies**：Self-contained macro implementation

## 📦 Installation

```toml
[dependencies]
dejavu-macros = "0.1"
```

## 🚀 Usage Examples

### Template Macro

```rust
use dejavu_macros::template;

#[template]
fn greeting(name: &str) -> String {
    "Hello, <% name %>!"
}

fn main() {
    let result = greeting("World");
    println!("{}", result); // Hello, World!
}
```

### Inline Templates

```rust
use dejavu_macros::inline_template;

fn main() {
    let name = "Alice";
    let result = inline_template!("Hello, <% name %>!");
    println!("{}", result); // Hello, Alice!
}
```

### Complex Templates

```rust
use dejavu_macros::template;

#[template]
fn user_profile(name: &str, age: u32, is_admin: bool) -> String {
    r#"
    <div class="profile">
        <h1><% name %></h1>
        <p>Age: <% age %></p>
        <% if is_admin %>
        <p>Admin: Yes</p>
        <% else %>
        <p>Admin: No</p>
        <% endif %>
    </div>
    "#
}

fn main() {
    let result = user_profile("Bob", 30, true);
    println!("{}", result);
}
```

## 🏗️ Architecture

`dejavu-macros` uses the Rust procedural macro system:

- **Compile-Time Processing**：Templates processed at compile time
- **Code Generation**：Generates optimized Rust code from templates
- **Syntax Validation**：Early error detection during compilation

## 🤝 Contributing

We welcome contributions! Whether it's bug reports, feature requests, or code, help us build the best macro system for
template engines. Check our [issues](https://github.com/oovm/dejavu-engine/issues) or submit a PR.

## 📄 License

This project is licensed under the [MPL-2.0](https://www.mozilla.org/en-US/MPL/2.0/) license.
