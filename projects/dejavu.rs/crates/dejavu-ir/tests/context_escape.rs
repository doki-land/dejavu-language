//! 上下文敏感转义（Rust oracle）。
//!
//! 合同（`dejavu-runtime-topology.md` §0.2）：仅 HTML text 上下文自动转义；
//! `escape_attr` / `escape_url` / `escape_js` / `escape_css` 是 `Pure` filter，
//! 输出是普通字符串，管道自动转义仍生效（双重转义是可见缺陷不是安全洞）。
//! `escape_url` 拒绝 allowlist 外 scheme 与 protocol-relative 引用：值归
//! null、渲染为空串，并向宿主 `WarningSink` 写入
//! `dejavu.template.url-scheme-rejected`。`safe` 只标记 HTML text 安全，
//! 跨上下文不可互换。

use dejavu_ir::{ArgValue, Diagnostic, IrDocument, RenderOptions, WarningSink, codes, default_language, render_ir_opts};
use serde_json::Value;

fn tpl_ir(children: Vec<serde_json::Value>) -> IrDocument {
    let doc_json = serde_json::json!({
        "irVersion": "v0",
        "language": default_language(),
        "body": {
            "type": "Template",
            "children": children,
        }
    });
    serde_json::from_value(doc_json).expect("parse ir json")
}

fn interpolation(expr: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "type": "Interpolation",
        "expression": expr,
        "trim": "none",
        "raw": false,
    })
}

fn literal(s: &str) -> serde_json::Value {
    serde_json::json!({ "type": "Expr.Literal", "value": s })
}

fn pipe(inner: serde_json::Value, filter: &str) -> serde_json::Value {
    serde_json::json!({ "type": "Expr.Pipe", "expression": inner, "filter": filter, "arguments": [] })
}

fn render_out(expr: serde_json::Value) -> String {
    let doc = tpl_ir(vec![interpolation(expr)]);
    render_ir_opts(&doc, &Value::Object(serde_json::Map::new()), RenderOptions::default(), None).expect("render")
}

fn render_warn(expr: serde_json::Value) -> (String, Vec<Diagnostic>) {
    let doc = tpl_ir(vec![interpolation(expr)]);
    let warnings = WarningSink::new();
    let opts = RenderOptions { warnings: Some(&warnings), ..Default::default() };
    let out = render_ir_opts(&doc, &Value::Object(serde_json::Map::new()), opts, None).expect("render");
    (out, warnings.take())
}

fn arg_str(d: &Diagnostic, key: &str) -> String {
    match d.args.get(key) {
        Some(ArgValue::String(s)) => s.clone(),
        other => panic!("expected string arg `{key}`, got {other:?}"),
    }
}

// ===========================================================================
// escape_attr
// ===========================================================================

#[test]
fn escape_attr_escapes_five_chars() {
    // 用 safe 隔离出 escape_attr 自身的转义集（`& < > " '`）。
    assert_eq!(
        render_out(pipe(pipe(literal(r#"<a href="x">'</a>"#), "escape_attr"), "safe")),
        "&lt;a href=&quot;x&quot;&gt;&#39;&lt;/a&gt;"
    );
}

#[test]
fn escape_attr_is_pure_so_pipeline_escapes_again() {
    // 组合规则：escape_attr 输出是普通字符串，自动转义仍生效（双重转义）。
    assert_eq!(render_out(pipe(literal("<b>"), "escape_attr")), "&amp;lt;b&amp;gt;");
}

#[test]
fn escape_attr_with_safe_is_single_escaped() {
    // 仅当输出已是完整 HTML text 才配 safe；这里 safe 跳过自动转义。
    assert_eq!(render_out(pipe(pipe(literal("<b>"), "escape_attr"), "safe")), "&lt;b&gt;");
}

// ===========================================================================
// escape_url
// ===========================================================================

#[test]
fn escape_url_allows_https_and_percent_encodes_space() {
    let (out, warnings) = render_warn(pipe(literal("https://example.com/a b?q=1&x=2"), "escape_url"));
    assert!(warnings.is_empty());
    // percent-encode 保留 `&`，随后自动转义把它变成 `&amp;`（属性内浏览器还原）。
    assert_eq!(out, "https://example.com/a%20b?q=1&amp;x=2");
}

#[test]
fn escape_url_allows_mailto_and_relative_paths() {
    for input in ["mailto:a@b.c", "/docs/a.md", "?q=1", "#frag", "page.html", ""] {
        let (out, warnings) = render_warn(pipe(literal(input), "escape_url"));
        assert!(warnings.is_empty(), "`{input}` must be allowed");
        assert_eq!(out, input);
    }
}

#[test]
fn escape_url_scheme_match_is_case_insensitive() {
    let (out, warnings) = render_warn(pipe(literal("HTTPS://x"), "escape_url"));
    assert!(warnings.is_empty());
    assert_eq!(out, "HTTPS://x");
}

#[test]
fn escape_url_rejects_javascript_scheme_with_warning() {
    let (out, warnings) = render_warn(pipe(literal("javascript:alert(1)"), "escape_url"));
    assert_eq!(out, "");
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].code, codes::URL_SCHEME_REJECTED);
    assert_eq!(arg_str(&warnings[0], "scheme"), "javascript");
    assert_eq!(arg_str(&warnings[0], "url"), "javascript:alert(1)");
}

#[test]
fn escape_url_rejects_protocol_relative_with_empty_scheme() {
    let (out, warnings) = render_warn(pipe(literal("//evil.com/x"), "escape_url"));
    assert_eq!(out, "");
    assert_eq!(warnings.len(), 1);
    assert_eq!(arg_str(&warnings[0], "scheme"), "");
    assert_eq!(arg_str(&warnings[0], "url"), "//evil.com/x");
}

#[test]
fn escape_url_rejects_data_scheme() {
    let (out, warnings) = render_warn(pipe(literal("data:text/html,<script>"), "escape_url"));
    assert_eq!(out, "");
    assert_eq!(arg_str(&warnings[0], "scheme"), "data");
}

#[test]
fn escape_url_without_sink_still_renders_empty() {
    // 宿主不传 sink 时警告丢弃，但拒绝语义（值归 null）不变。
    assert_eq!(render_out(pipe(literal("javascript:alert(1)"), "escape_url")), "");
}

#[test]
fn escape_url_encodes_percent_and_utf8() {
    let (out, warnings) = render_warn(pipe(literal("/a%20b"), "escape_url"));
    assert!(warnings.is_empty());
    assert_eq!(out, "/a%2520b");

    let (out, warnings) = render_warn(pipe(literal("/é"), "escape_url"));
    assert!(warnings.is_empty());
    assert_eq!(out, "/%C3%A9");
}

#[test]
fn escape_url_not_a_scheme_when_head_invalid() {
    // `java script:` 的 head 含空格，不构成 scheme：按相对路径放行并编码。
    let (out, warnings) = render_warn(pipe(literal("java script:alert(1)"), "escape_url"));
    assert!(warnings.is_empty());
    assert_eq!(out, "java%20script:alert(1)");
}

// ===========================================================================
// escape_js
// ===========================================================================

#[test]
fn escape_js_wraps_in_quotes_and_escapes_quotes() {
    // 引号突破：输入含 `"`，输出是完整 JS 字面量。
    let out = render_out(pipe(literal(r#""; alert(1); ""#), "escape_js"));
    assert_eq!(out, "&quot;\\&quot;; alert(1); \\&quot;&quot;");
}

#[test]
fn escape_js_escapes_control_and_line_separators() {
    let out = render_out(pipe(literal("a\nb\u{2028}c\u{0000}d"), "escape_js"));
    assert_eq!(out, "&quot;a\\nb\\u2028c\\u0000d&quot;");
}

#[test]
fn escape_js_keeps_html_chars_for_pipeline() {
    // `< > & '` 不属于 JS 字面量转义集，交给自动转义。
    let out = render_out(pipe(literal("<&'>"), "escape_js"));
    assert_eq!(out, "&quot;&lt;&amp;&#39;&gt;&quot;");
}

#[test]
fn escape_js_with_safe_is_not_html_escaped() {
    // safe 只标记 HTML text 安全：escape_js 输出配 safe 后原样透出，
    // 这正是「跨上下文不可互换」的反例（作者必须自己保证 text 安全）。
    let out = render_out(pipe(pipe(literal("<b>"), "escape_js"), "safe"));
    assert_eq!(out, "\"<b>\"");
}

// ===========================================================================
// escape_css
// ===========================================================================

#[test]
fn escape_css_escapes_semicolon_and_colon_with_terminating_space() {
    let out = render_out(pipe(literal("red;color:blue"), "escape_css"));
    assert_eq!(out, r"red\3b color\3a blue");
}

#[test]
fn escape_css_doubles_backslash_and_keeps_non_ascii() {
    let out = render_out(pipe(literal(r"a\bé"), "escape_css"));
    assert_eq!(out, r"a\\bé");
}

#[test]
fn escape_css_escapes_newline_as_shortest_hex() {
    let out = render_out(pipe(literal("a\nb"), "escape_css"));
    assert_eq!(out, "a\\a b");
}

// ===========================================================================
// 默认行为零变化
// ===========================================================================

#[test]
fn default_interpolation_still_automatically_escaped() {
    assert_eq!(render_out(literal("<b>&\"'")), "&lt;b&gt;&amp;&quot;&#39;");
}

#[test]
fn new_filters_are_registered_as_pure() {
    for name in ["escape_attr", "escape_url", "escape_js", "escape_css"] {
        let spec = dejavu_ir::registry::filter_spec(name).unwrap_or_else(|| panic!("`{name}` missing"));
        assert_eq!(spec.capability, dejavu_ir::Capability::Pure);
        assert_eq!(spec.min_args, 0);
        assert_eq!(spec.max_args, 0);
    }
}
