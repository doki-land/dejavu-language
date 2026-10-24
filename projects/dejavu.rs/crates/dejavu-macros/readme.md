# DejaVu Macros — 占位状态

> **当前状态：占位实现。** 完整的 Askama 式编译期代码生成尚未恢复。

`dejavu-macros` 曾通过 Oak（dejavu 早期 AOT 编译器）提供编译期模板处理。
该路径已被移除，当前只保留了最小占位接口。

## 当前可用

### `#[derive(Template)]`

生成一个 `render_hint()` 静态方法，提示使用运行时 API：

```rust
use dejavu_macros::Template;

#[derive(Template)]
#[template(path = "templates/hello.dejavu")]
struct HelloTemplate;

fn main() {
    assert_eq!(HelloTemplate::render_hint(), "use dejavu::Dejavu::render_source");
}
```

### 运行时替代

在 Askama 式 AOT 恢复前，直接使用 `dejavu::Dejavu::render_source`：

```rust
use dejavu::Dejavu;

fn render() -> String {
    Dejavu::render_source(
        "Hello, <% name %>!",
        &serde_json::json!({"name": "World"}),
    ).unwrap()
}
```

## 待恢复：Askama 式编译期代码生成

完整的 `#[derive(Template)]` 和 `template!` 宏将在 IR-based 重写完成后恢复。
参考 Askama 的设计：

1. **类型模型**：模板如何绑定 Rust struct 字段，由 Rust 类型系统检查
2. **模板依赖**：`include`、`extends` 等在编译时如何解析和触发重编译
3. **语义一致性**：IR renderer 与 AOT 后端产出必须完全等价
4. **诊断体验**：语法错误和类型错误在构建期定位到模板源文件

设计草案和进度见 issue tracker。

## 为什么 Oak 路径被移除

dejavu 选择先完成 IR + 跨宿主 renderer，确保语义合同冻结后再恢复 Rust AOT。
这样 Askama 式后端可以直接复用已验证的 IR，而不是另造一套模板语义。
