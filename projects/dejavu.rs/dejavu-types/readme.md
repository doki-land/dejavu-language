# 🚀 DejaVu Types

[![Crates.io](https://img.shields.io/crates/v/dejavu-types.svg)](https://crates.io/crates/dejavu-types)
[![Documentation](https://docs.rs/dejavu-types/badge.svg)](https://docs.rs/dejavu-types)

**Core Implementation for Template Engines.** `dejavu-types` is the foundational library for the DejaVu template engine,
providing both AOT (Ahead-Of-Time) compilation and Dyn (Dynamic) interpretation modes for maximum flexibility and
performance.

## ✨ Core Features

### AOT Mode

- **⚡ Maximum Performance**：Compile templates to native code (Rust, TypeScript, JavaScript)
- **🛠️ Zero Runtime Overhead**：No interpretation at runtime
- **🎯 Type Safety**：Compile-time validation and type checking
- **📦 Optimized Output**：Generated code tailored to target language

### Dyn Mode

- **🔄 Dynamic Execution**：Runtime interpretation for maximum flexibility
- **📱 Interactive Development**：Ideal for REPL and development environments
- **🎨 Dynamic Template Loading**：Load templates at runtime
- **🔧 Runtime Evaluation**：Evaluate expressions dynamically

### Core Capabilities

- **🌳 High-Fidelity AST**：Comprehensive abstract syntax tree
- **🔍 Robust Parser**：Lexer and parser for template syntax
- **🛠️ Extensible Compiler**：Pluggable backend system
- **📋 Rich Error System**：Detailed error messages with context
- **🔗 Serde Integration**：Seamless serialization support

## 📦 Installation

```toml
[dependencies]
dejavu-types = "0.1"
```

## 🚀 Usage Examples

### AOT Compilation

```rust
use dejavu_types::{DejavuCompiler, CompileTarget, CompileOptions};

fn main() {
    let template = "Hello, {{ name }}!";
    let mut compiler = DejavuCompiler::new();
    
    let result = compiler.compile(
        template,
        CompileTarget::Rust,
        CompileOptions::default()
    );
    
    match result {
        Ok(code) => println!("Generated code: {}", code),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Dynamic VM

```rust
use dejavu_types::{DejavuVM, Context, Value};

fn main() {
    let template = "Hello, {{ name }}!";
    let mut vm = DejavuVM::new();
    
    let mut context = Context::new();
    context.set("name", Value::String("World".into()));
    
    let result = vm.render(template, &context);
    
    match result {
        Ok(output) => println!("Output: {}", output),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Advanced Template

```rust
use dejavu_types::{parse, Template, Context, Value};

fn main() {
    let template = r#"
    <h1>{{ title }}</h1>
    <ul>
    <% for item in items %>
        <li>{{ item }}</li>
    <% endfor %>
    </ul>
    "#;
    
    // Parse template
    let template = parse(template).unwrap();
    
    // Create context
    let mut context = Context::new();
    context.set("title", Value::String("My List".into()));
    context.set("items", Value::Array(vec![
        Value::String("Item 1".into()),
        Value::String("Item 2".into()),
        Value::String("Item 3".into()),
    ]));
    
    // Render template
    let mut output = String::new();
    template.render(&context, &mut output).unwrap();
    
    println!("{}", output);
}
```

## 📁 Module Structure

- **`ast`**：Abstract syntax tree definitions
- **`parser`**：Lexer and parser implementation
- **`aot`**：AOT compiler with multiple backends
- **`dyn_vm`**：Dynamic virtual machine for runtime interpretation
- **`runtime`**：Runtime types and utilities
- **`error`**：Error types and handling
- **`filters`**：Built-in filter functions

## 🏗️ Architecture

```
dejavu-types (core implementation)
  ↑
dejavu-engine (frontend)
  ↑
dejavu (runtime)
```

- **Core Layer**：Complete template engine implementation
- **Frontend Integration**：Used by `dejavu-engine` for user-friendly API
- **Runtime Support**：Provides types and utilities for `dejavu`

## 🤝 Contributing

We welcome contributions! Whether it's bug reports, feature requests, or code, help us build the best core
implementation for template engines. Check our [issues](https://github.com/oovm/dejavu-engine/issues) or submit a PR.

## 📄 License

This project is licensed under the [MPL-2.0](https://www.mozilla.org/en-US/MPL/2.0/) license.
