# 🚀 DejaVu Tools (Rust CLI)

[![Crates.io](https://img.shields.io/crates/v/dejavu-tools.svg)](https://crates.io/crates/dejavu-tools)
[![Documentation](https://docs.rs/dejavu-tools/badge.svg)](https://docs.rs/dejavu-tools)

**High-Performance CLI for Template Engines.** `dejavu-tools` is a blazingly fast command-line interface for the DejaVu
template engine, providing comprehensive tooling for template compilation, validation, and formatting.

## ✨ Core Features

- **⚡ Blazing Fast**：Native Rust implementation for maximum performance
- **🔧 Comprehensive Tooling**：Complete toolchain for template processing
- **🎯 Multi-Target Support**：Compile to Rust, TypeScript, and JavaScript
- **🛠️ Detailed Diagnostics**：Rich error messages with line numbers and context
- **📦 Batch Processing**：Handle multiple templates efficiently

## 📦 Installation

```bash
# Install from crates.io
cargo install dejavu-tools

# Check installation
dejavu --help
```

## 🚀 Command-Line Usage

### Compile Templates

```bash
# Compile to Rust
dejavu compile template.dejavu -o output.rs

# Compile to TypeScript
dejavu compile template.dejavu -o output.ts --target typescript

# Compile to JavaScript
dejavu compile template.dejavu -o output.js --target javascript
```

### Validate Templates

```bash
# Check syntax validity
dejavu validate template.dejavu

# Validate multiple templates
dejavu validate templates/*.dejavu
```

### Format Templates

```bash
# Format template in-place
dejavu format template.dejavu

# Format with custom indentation
dejavu format template.dejavu --indent 2
```

### Help and Documentation

```bash
# Get help
dejavu --help

# Command-specific help
dejavu compile --help
```

## 📁 Command Reference

| Command    | Description                         |
|------------|-------------------------------------|
| `compile`  | Compile template to target language |
| `validate` | Validate template syntax            |
| `format`   | Format template file                |
| `help`     | Show help information               |

## 🏗️ Architecture

```
dejavu-tools (CLI)
  ↑
dejavu-engine (frontend)
  ↑
dejavu-types (core implementation)
  ↑
dejavu (runtime)
```

- **CLI Layer**：User-friendly command-line interface
- **Frontend Integration**：Leverages `dejavu-engine` for core functionality
- **Core Processing**：Uses `dejavu-types` for parsing and compilation

## 🤝 Contributing

We welcome contributions! Whether it's bug reports, feature requests, or code, help us build the best CLI for template
engines. Check our [issues](https://github.com/oovm/dejavu-engine/issues) or submit a PR.

## 📄 License

This project is licensed under the [MPL-2.0](https://www.mozilla.org/en-US/MPL/2.0/) license.
