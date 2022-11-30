#![cfg(test)]
use dejavu::{
    escaper::{Html, Text},
    *,
};

#[test]
fn test_html_escaper() {
    let mut output = String::new();
    Html.write_escaped(&mut output, "Hello, world!").unwrap();
    assert_eq!(output, "Hello, world!");

    let mut output = String::new();
    Html.write_escaped(&mut output, "<script>alert('xss')</script>").unwrap();
    assert_eq!(output, "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");

    let mut output = String::new();
    Html.write_escaped(&mut output, "<div class=\"test\">").unwrap();
    assert_eq!(output, "&lt;div class=&quot;test&quot;&gt;");

    let mut output = String::new();
    Html.write_escaped(&mut output, "a & b && c").unwrap();
    assert_eq!(output, "a &amp; b &amp;&amp; c");
}

#[test]
fn test_text_escaper() {
    let mut output = String::new();
    Text.write_escaped(&mut output, "Hello, world!").unwrap();
    assert_eq!(output, "Hello, world!");

    let mut output = String::new();
    Text.write_escaped(&mut output, "<script>alert('xss')</script>").unwrap();
    assert_eq!(output, "<script>alert('xss')</script>");

    let mut output = String::new();
    Text.write_escaped(&mut output, "any text & < > ' \"").unwrap();
    assert_eq!(output, "any text & < > ' \"");
}

#[test]
fn test_escape_function() {
    let mut config = HtmlEscapeConfig::default();
    config.enabled = true;
    let escaped = escape_with_config("Hello <b>world</b>", Html, config);
    assert_eq!(escaped.to_string(), "Hello &lt;b&gt;world&lt;/b&gt;");

    let escaped = escape("Plain text", Text);
    assert_eq!(escaped.to_string(), "Plain text");
}

#[test]
fn test_escape_with_config() {
    let mut config = HtmlEscapeConfig::default();
    config.enabled = true;

    let escaped = escape_with_config("Hello <b>world</b>", Html, config.clone());
    assert_eq!(escaped.to_string(), "Hello &lt;b&gt;world&lt;/b&gt;");

    config.enabled = false;
    let escaped = escape_with_config("Hello <b>world</b>", Html, config);
    assert_eq!(escaped.to_string(), "Hello <b>world</b>");
}

#[test]
fn test_escape_display_dangerous() {
    let mut config = HtmlEscapeConfig::default();
    config.enabled = true;
    let display = EscapeDisplay::dangerous_with_config("<b>test</b>", Html, config);
    assert_eq!(display.to_string(), "&lt;b&gt;test&lt;/b&gt;");
}

#[test]
fn test_escape_display_safe() {
    let display = EscapeDisplay::safe("<b>test</b>", Html);
    assert_eq!(display.to_string(), "<b>test</b>");
}

#[test]
fn test_escape_display_mark_safe() {
    let display = EscapeDisplay::dangerous("<b>test</b>", Html);
    let safe_display = display.mark_safe();
    assert_eq!(safe_display.to_string(), "<b>test</b>");
}

#[test]
fn test_escape_display_with_config() {
    let mut config = HtmlEscapeConfig::default();
    config.enabled = true;

    let display = EscapeDisplay::dangerous_with_config("<b>test</b>", Html, config.clone());
    assert_eq!(display.to_string(), "&lt;b&gt;test&lt;/b&gt;");

    let display_safe = EscapeDisplay::safe_with_config("<b>test</b>", Html, config);
    assert_eq!(display_safe.to_string(), "<b>test</b>");
}

#[test]
fn test_escape_display_with_config_method() {
    let mut config = HtmlEscapeConfig::default();
    config.enabled = false;

    let display = EscapeDisplay::dangerous("<b>test</b>", Html).with_config(config);
    assert_eq!(display.to_string(), "<b>test</b>");
}

#[test]
fn test_safe_html_escape() {
    let mut config = HtmlEscapeConfig::default();
    config.enabled = true;
    config.escape_content = true;

    let result = safe_html_escape("<b>test</b>", Html, config.clone());
    assert_eq!(result, "&lt;b&gt;test&lt;/b&gt;");

    config.escape_content = false;
    let result = safe_html_escape("<b>test</b>", Html, config.clone());
    assert_eq!(result, "<b>test</b>");

    config.enabled = false;
    config.escape_content = true;
    let result = safe_html_escape("<b>test</b>", Html, config);
    assert_eq!(result, "<b>test</b>");
}

#[test]
fn test_safe_html_attribute_escape() {
    let mut config = HtmlEscapeConfig::default();
    config.enabled = true;
    config.escape_attributes = true;

    let result = safe_html_attribute_escape("<b>test</b>", Html, config.clone());
    assert_eq!(result, "&lt;b&gt;test&lt;/b&gt;");

    config.escape_attributes = false;
    let result = safe_html_attribute_escape("<b>test</b>", Html, config.clone());
    assert_eq!(result, "<b>test</b>");

    config.enabled = false;
    config.escape_attributes = true;
    let result = safe_html_attribute_escape("<b>test</b>", Html, config);
    assert_eq!(result, "<b>test</b>");
}

#[test]
fn test_html_escape_config_default() {
    let config = HtmlEscapeConfig::default();
    assert!(!config.enabled);
    assert!(!config.escape_attributes);
    assert!(!config.escape_content);
}

#[test]
fn test_html_escape_config_clone() {
    let mut config = HtmlEscapeConfig::default();
    config.enabled = true;
    config.escape_attributes = true;
    config.escape_content = true;

    let cloned = config.clone();
    assert_eq!(config.enabled, cloned.enabled);
    assert_eq!(config.escape_attributes, cloned.escape_attributes);
    assert_eq!(config.escape_content, cloned.escape_content);
}

#[test]
fn test_html_escape_config_debug() {
    let config = HtmlEscapeConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("HtmlEscapeConfig"));
}
