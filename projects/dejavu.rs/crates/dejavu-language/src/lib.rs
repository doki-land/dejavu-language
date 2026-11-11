//! 兼容再导出：Dejavu 语法前端现由 `document.rs` 的 `template.input.dejavu` 提供。
//!
//! 本仓不再维护第二份 parser；对外名称与 `ParseError` 形状保持不变。

pub use document_template_input_dejavu::language::*;
