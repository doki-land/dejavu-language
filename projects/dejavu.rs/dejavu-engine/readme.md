# 🚀 DejaVu Engine

[![Crates.io](https://img.shields.io/crates/v/dejavu-engine.svg)](https://crates.io/crates/dejavu-engine)
[![Documentation](https://docs.rs/dejavu-engine/badge.svg)](https://docs.rs/dejavu-engine)

**User-Friendly Frontend for Template Engines.** `dejavu-engine` provides a clean, intuitive API for the DejaVu template
engine, making it easy to parse, compile, and render templates with minimal boilerplate.

## ✨ Core Features

- **🔧 Intuitive API**：Clean, fluent interface for template operations
- **⚡ Seamless Integration**：Built on top of `dejavu-types` core implementation
- **🎯 Full Type Support**：Complete type re-exports for easy access
- **🛠️ Programmatic Building**：Build templates programmatically with the fluent builder API
- **🔄 AOT Compilation**：Efficient ahead-of-time compilation to multiple targets

## 📦 Installation

```toml
[dependencies]
dejavu-engine = "0.1"
```

## 🚀 Usage Examples

### Basic Parsing

```rust
use dejavu_engine::{parse, Template};

fn main() -> dejavu_engine::DejavuResult<()> {
    // Parse a template from string
    let template = parse("Hello, {{ name }}!")?;
    
    println!("Template parsed successfully");
    Ok(())
}
```

### Fluent Builder API

```rust
use dejavu_engine::{DejavuBuilder, TemplateMode};

fn main() -> dejavu_engine::DejavuResult<()> {
    // Build template programmatically
    let template = DejavuBuilder::empty()
        .mode(TemplateMode::Text)
        .declaration("greeting")
        .text("Hello, ")
        .var("name")
        .text("!")
        .build()?;
    
    println!("Template built successfully");
    Ok(())
}
```

### AOT Compilation

```rust
use dejavu_engine::{DejavuCompiler, CompileTarget, CompileOptions};

fn main() -> dejavu_engine::DejavuResult<()> {
    // Configure compiler
    let mut options = CompileOptions::default();
    options.target = CompileTarget::Rust;
    
    // Compile template
    let compiler = DejavuCompiler::new(options);
    let result = compiler.compile_str("Hello, {{ name }}!")?;
    
    println!("Generated code:\n{}", result.code);
    Ok(())
}
```

## 📁 Module Structure

- **`builder`**：Fluent template builder API
- **`compiler`**：AOT compiler wrapper
- **`error`**：Unified error types

## 🔄 Re-Exported Types

`dejavu-engine` re-exports all major types from `dejavu-types` for convenience:

- **AST Types**：`Template`, `Element`, `Expr`, `IfBlock`, `ForBlock`
- **Parsers**：`parse`, `parse_with_path`, `parse_with_mode`
- **Runtime**：`Context`, `Value`, `Template` trait
- **Errors**：`ParseError`, `CompileError`

## 🏗️ Architecture

```
dejavu (runtime) ──┐
                  │
                  ▼
dejavu-engine (frontend) ──→ dejavu-types (core implementation)
```

- **Frontend Layer**：User-friendly API for common operations
- **Core Integration**：Leverages `dejavu-types` for heavy lifting
- **Unified Errors**：Consistent error handling across all operations

## 🤝 Contributing

We welcome contributions! Whether it's bug reports, feature requests, or code, help us build the best template engine
frontend. Check our [issues](https://github.com/oovm/dejavu-engine/issues) or submit a PR.

## 📄 License

This project is licensed under the [MPL-2.0](https://www.mozilla.org/en-US/MPL/2.0/) license.
