//! CLI 友好的诊断格式化。
//!
//! 把 [`dejavu_diagnostic::Diagnostic`] 格式化成类似 `rustc`/`eslint` 风格的可读输出，
//! 用于 Build/Check 命令的终端展示。

use dejavu_diagnostic::Diagnostic;
use std::path::{Path, PathBuf};

/// 把 UTF-8 byte offset 转成 1-based (line, column)。
///
/// 返回 `(line, column)`；offset 越界时 clamp 到源文本末尾。
pub fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let off = offset.min(bytes.len());

    // line = 1-based，遇到多少个 `\n` 在 off 之前就 line = count + 1
    let mut line = 1usize;
    let mut last_line_start = 0usize;
    for (i, &b) in bytes.iter().enumerate().take(off) {
        if b == b'\n' {
            line += 1;
            last_line_start = i + 1;
        }
    }
    let col = off - last_line_start + 1; // 1-based
    (line, col)
}

/// 单个诊断的格式化输出（一行或多行，不自带换行）。
///
/// 格式：
/// ```text
/// error[dejavu.parse.unclosed-block]: unclosed code block
///   → templates/foo.dejavu:3:12
/// ```
pub fn format_diagnostic(d: &Diagnostic, source: Option<&str>) -> String {
    let sv = severity_label(d.severity);
    let mut out = String::new();
    out.push_str(sv);
    out.push_str("[");
    out.push_str(&d.code);
    out.push_str("]: ");
    if let Some(msg) = &d.message {
        out.push_str(msg);
    } else {
        out.push_str(&d.code);
    }
    out.push('\n');

    // 路径 + span
    if let Some(path) = d.path.as_deref() {
        out.push_str("   → ");
        out.push_str(path);
        if let (Some(span), Some(src)) = (d.span, source) {
            let (line, col) = byte_offset_to_line_col(src, span.start);
            out.push_str(&format!(":{line}:{col}"));
            if let Some(len) = span.len().checked_sub(1) {
                if len > 0 {
                    let (eline, ecol) = byte_offset_to_line_col(src, span.end);
                    out.push_str(&format!("-{eline}:{ecol}"));
                }
            }
        } else if let Some(span) = d.span {
            out.push_str(&format!(" @ {}-{}", span.start, span.end));
        }
        out.push('\n');
    } else if let Some(span) = d.span {
        out.push_str(&format!("   → @ {}-{}\n", span.start, span.end));
    }

    // args（如果有，以 JSON 附在末尾，便于调试）
    if !d.args.is_empty() {
        if let Ok(json) = serde_json::to_string(&d.args) {
            out.push_str("   args: ");
            out.push_str(&json);
            out.push('\n');
        }
    }

    out
}

fn severity_label(s: dejavu_diagnostic::Severity) -> &'static str {
    match s {
        dejavu_diagnostic::Severity::Error => "error",
        dejavu_diagnostic::Severity::Warning => "warning",
        dejavu_diagnostic::Severity::Info => "info",
        dejavu_diagnostic::Severity::Debug => "debug",
    }
}

/// 格式化诊断集合，附带统计行。
///
/// `source_map` 用于按 path 查源码以计算 line:col；可以为空 map（此时不输出具体行列）。
pub fn format_diagnostics(
    diagnostics: &[Diagnostic],
    source_map: &std::collections::HashMap<String, String>,
) -> String {
    let mut out = String::new();
    for d in diagnostics {
        let src = d.path.as_ref().and_then(|p| source_map.get(p));
        out.push_str(&format_diagnostic(d, src.map(|s| s.as_str())));
    }
    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == dejavu_diagnostic::Severity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == dejavu_diagnostic::Severity::Warning)
        .count();
    if errors > 0 || warnings > 0 {
        out.push_str(&format!(
            "\n{} error(s), {} warning(s)\n",
            errors, warnings
        ));
    }
    out
}

/// 把 `dejavu_language::ParseError` 转成 `Diagnostic`（stage = Parse）。
pub fn parse_error_to_diagnostic(err: &dejavu_language::ParseError) -> Diagnostic {
    Diagnostic::new(
        dejavu_diagnostic::Severity::Error,
        err.code_or_default(),
        dejavu_diagnostic::Stage::Parse,
    )
    .path(err.file.clone())
    .span(err.span.start, err.span.end)
    .message(err.message.clone())
    .arg("label", err.label.clone())
}

/// 可选：把相对路径相对于 source root 显示，让输出更短。
pub fn relativize(path: &str, root: &Path) -> String {
    let p = PathBuf::from(path);
    p.strip_prefix(root)
        .map(|rel| rel.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string())
}

// ---------------------------------------------------------------------------
// 为了让 diag.rs 自包含可测试，构造一个最小 Diagnostic 用例
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use dejavu_diagnostic::{Diagnostic as D, Severity, Span, Stage};

    #[test]
    fn byte_offset_converts_to_line_col() {
        let src = "hello\nworld\n!\n";
        assert_eq!(byte_offset_to_line_col(src, 0), (1, 1));
        assert_eq!(byte_offset_to_line_col(src, 5), (1, 6)); // end of "hello"
        assert_eq!(byte_offset_to_line_col(src, 6), (2, 1)); // start of "world"
        assert_eq!(byte_offset_to_line_col(src, 11), (2, 6)); // end of "world"
        assert_eq!(byte_offset_to_line_col(src, 12), (3, 1)); // "!"
        assert_eq!(byte_offset_to_line_col(src, 200), (4, 1)); // clamp at end
    }

    #[test]
    fn format_includes_severity_code_path_span() {
        let src = "a = 1\nif x {{ }}";
        // 计算：offset 0..5 = "a = 1\n"（6 字节），offset 6 = 'i'（line 2, col 1）
        let d = D::new(Severity::Error, "dejavu.parse.unexpected-token", Stage::Parse)
            .path("templates/foo.dejavu")
            .span(11, 13) // 对应 "{{" — offset 11='{', 12='{', 13=' '
            .message("unexpected token `{{`");

        let out = format_diagnostic(&d, Some(src));
        assert!(out.contains("error[dejavu.parse.unexpected-token]"));
        assert!(out.contains("unexpected token `{{`"));
        assert!(out.contains("templates/foo.dejavu:2:6")); // offset 11 在 line 2, col 6（"if x " = 5 字符 + 1）
    }

    #[test]
    fn parse_error_roundtrip() {
        let perr = dejavu_language::ParseError::at(
            "hi <% x %>",
            "t.dejavu",
            3,
            2,
            "oops",
            "here",
        )
        .with_code(dejavu_diagnostic::codes::PARSE_UNEXPECTED_TOKEN);

        let d = parse_error_to_diagnostic(&perr);
        assert_eq!(d.severity, Severity::Error);
        assert_eq!(d.code, dejavu_diagnostic::codes::PARSE_UNEXPECTED_TOKEN);
        assert_eq!(d.path.as_deref(), Some("t.dejavu"));
        assert_eq!(d.span, Some(Span::new(3, 5)));
        assert_eq!(d.message.as_deref(), Some("oops"));
    }
}
