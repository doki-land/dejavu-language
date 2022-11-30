# Rust 公开表面

本页供 Rust facade 维护者核对公开 API。宿主采用前必须查看[实现状态](../compatibility.md)。

Rust facade 应提供与规范操作等价的 parse、render、render source 和 check：

```rust
use dejavu::Dejavu;
use serde_json::json;

fn main() {
    let output = Dejavu::render_source(
        "Hello, <% account.name %>!",
        &json!({ "account": { "name": "Mira" } }),
    ).expect("render");
    println!("{output}");
}
```

内部 crate 可以按 Rust workspace 需要拆分，但应用 facade 不应要求用户了解 parser、IR 和 renderer 的内部依赖图。占位宏或未通过 conformance 的 AOT 能力不得写成正式接入方式。
