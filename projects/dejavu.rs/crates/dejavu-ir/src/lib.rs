//! 兼容再导出：Dejavu IR、渲染与 catalog 现由 `document.rs` 的
//! `template.input.dejavu` 提供。
//!
//! 本仓不再维护第二份实现；对外名称、诊断码与渲染行为保持不变。

pub use template_input_dejavu::*;
