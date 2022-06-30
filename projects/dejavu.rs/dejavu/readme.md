# 🚀 DejaVu Runtime

[![Crates.io](https://img.shields.io/crates/v/dejavu.svg)](https://crates.io/crates/dejavu)
[![Documentation](https://docs.rs/dejavu/badge.svg)](https://docs.rs/dejavu)

**Minimal Runtime for Maximum Performance.** `dejavu` is a lightweight, `#![no_std]` compatible runtime library for the
DejaVu template engine, providing core rendering capabilities with zero dependencies.

## ✨ Core Features

- **⚡ Zero Overhead**：Minimal runtime footprint with no external dependencies
- **🛡️ `#![no_std]` Compatible**：Works in embedded and no-std environments
- **🔧 Essential Rendering Tools**：
    - `Template` trait：Core template rendering interface
    - `Escaper`：Text escaping utilities for HTML and text
    - `Looper`：Iteration and loop support
    - `EscapeDisplay`：Safe display interface for template variables

## 📦 Installation

```toml
[dependencies]
dejavu = "0.1"
```

## 🚀 Usage Examples

### Basic Rendering

```rust
use dejavu::{Template, EscapeDisplay};

// Implement the Template trait for your data struct
struct User {
    name: String,
    email: String,
}

impl Template for User {
    fn render(&self, writer: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(writer, "Hello, {}", self.name.escape_html())?;
        write!(writer, "Your email is: {}", self.email.escape_html())?;
        Ok(())
    }
}

fn main() {
    let user = User {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };
    
    let mut output = String::new();
    user.render(&mut output).unwrap();
    println!("{}", output);
}
```

### Using Escapers

```rust
use dejavu::Escaper;

fn main() {
    let html_escaper = Escaper::html();
    let text = "<script>alert('XSS')</script>";
    let escaped = html_escaper.escape(text);
    println!("Escaped: {}", escaped); // &lt;script&gt;alert('XSS')&lt;/script&gt;
}
```

## 🏗️ Architecture

`dejavu` provides the minimal runtime layer for the DejaVu ecosystem:

```
dejavu (runtime) ──┐
                  │
                  ▼
dejavu-engine (frontend) ──→ dejavu-types (core implementation)
```

- **Runtime Only**：Focused on rendering capabilities
- **No Compilation**：Parsing and compilation handled by `dejavu-engine`
- **Minimal API**：Clean, focused interface for template rendering

## 🤝 Contributing

We welcome contributions! Whether it's bug reports, feature requests, or code, help us build the best template engine
runtime. Check our [issues](https://github.com/oovm/dejavu-engine/issues) or submit a PR.

## 📄 License

This project is licensed under the [MPL-2.0](https://www.mozilla.org/en-US/MPL/2.0/) license.

