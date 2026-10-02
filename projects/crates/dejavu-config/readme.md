# 🚀 DejaVu Config

[![Crates.io](https://img.shields.io/crates/v/dejavu-config.svg)](https://crates.io/crates/dejavu-config)
[![Documentation](https://docs.rs/dejavu-config/badge.svg)](https://docs.rs/dejavu-config)

**Smart Configuration for Template Engines.** `dejavu-config` provides comprehensive configuration management for the
DejaVu template engine, supporting TOML, environment variables, and programmatic configuration.

## ✨ Core Features

- **📦 Multi-Source Configuration**：Supports TOML files, environment variables, and programmatic configuration
- **🔧 Flexible Overrides**：Environment variables can override file-based configurations
- **🎯 Type-Safe API**：Strongly-typed configuration structs with Serde support
- **⚡ Zero-Copy Parsing**：Efficient TOML parsing with toml_edit
- **🛠️ Extensible**：Easy to add custom configuration sources

## 📦 Installation

```toml
[dependencies]
dejavu-config = "0.1"
```

## 🚀 Usage Examples

### Basic Configuration

```rust
use dejavu_config::DejavuConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from default locations
    let config = DejavuConfig::load()?;
    
    // Access configuration values
    println!("Template directory: {:?}", config.template_dir);
    println!("Default target: {:?}", config.default_target);
    
    Ok(())
}
```

### Custom Configuration

```rust
use dejavu_config::{DejavuConfig, CompileTarget};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration programmatically
    let mut config = DejavuConfig::default();
    config.template_dir = Some("./templates".into());
    config.default_target = CompileTarget::Rust;
    config.optimize = true;
    
    // Save to file
    config.save("./dejavu.toml")?;
    
    Ok(())
}
```

## 📁 Configuration File Format

```toml
# dejavu.toml
[dejavu]
template_dir = "./templates"
default_target = "rust"
optimize = true

[dejavu.filters]
enable_builtins = true
```

## 🏗️ Architecture

`dejavu-config` uses the Figment library for configuration management:

- **Sources**：TOML files, environment variables, programmatic
- **Merging**：Intelligent merging with environment variable overrides
- **Validation**：Type-safe parsing with Serde

## 🤝 Contributing

We welcome contributions! Whether it's bug reports, feature requests, or code, help us build the best configuration
system for template engines. Check our [issues](https://github.com/oovm/dejavu-engine/issues) or submit a PR.

## 📄 License

This project is licensed under the [MPL-2.0](https://www.mozilla.org/en-US/MPL/2.0/) license.
