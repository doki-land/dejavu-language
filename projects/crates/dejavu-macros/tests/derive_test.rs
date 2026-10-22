//! Tests for the stub `#[derive(Template)]` placeholder.
//!
//! 完整的 Askama 式编译期代码生成尚未恢复；
//! 当前宏仅提供 `render_hint()` 提示使用运行时 `dejavu::Dejavu::render_source`。

use dejavu_macros::Template;

#[derive(Template)]
#[template(path = "templates/hello.dejavu")]
struct HelloTemplate;

#[test]
fn test_derive_produces_render_hint() {
    // 占位宏只生成 `render_hint()`，不生成真正的 `render()`。
    assert_eq!(HelloTemplate::render_hint(), "use dejavu::Dejavu::render_source");
}

#[test]
fn test_runtime_render_works() {
    // 用占位宏指向的同一模板，通过 runtime 验证渲染结果。
    let src = std::fs::read_to_string("templates/hello.dejavu").unwrap();
    let ctx = serde_json::json!({"name": "World"});

    let out = dejavu::Dejavu::render_source(&src, &ctx).unwrap();
    assert_eq!(out.trim(), "Hello, World!");
}

#[test]
fn test_runtime_render_html_escape() {
    // 默认应做 HTML 转义
    let src = std::fs::read_to_string("templates/hello.dejavu").unwrap();
    let ctx = serde_json::json!({"name": "<script>alert(1)</script>"});

    let out = dejavu::Dejavu::render_source(&src, &ctx).unwrap();
    assert!(out.contains("&lt;script&gt;"));
    assert!(!out.contains("<script>"));
}
