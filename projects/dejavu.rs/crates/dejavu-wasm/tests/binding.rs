//! WASM 语言桥的导出面合同测试。
//!
//! 所有调用都走 JSON 字符串，与 TS 包装层的实际用法一致。

use dejavu_wasm::{
    binding_contract_version, parse_ir, precompile_ir_single, render_ir_api, render_source_api, resolve_template_ref,
};
use serde_json::Value;

fn q(value: &str) -> String {
    serde_json::to_string(value).expect("string should serialize")
}

fn parse_json(raw: &str) -> Value {
    serde_json::from_str(raw).expect("response should be valid json")
}

fn diagnostics(response: &Value) -> &Vec<Value> {
    response["diagnostics"].as_array().expect("diagnostics array")
}

#[test]
fn binding_contract_version_is_stable() {
    assert_eq!(binding_contract_version(), "dejavu.wasm/1");
}

#[test]
fn parse_and_render_if_and_loop() {
    let source = "Hello <% name %>!<% if ok %> ON<% end if %><% loop item in items %>[<% item %>]<% end loop %>";
    let parsed = parse_json(&parse_ir(&format!(r#"{{"source":{},"file":"page.dejavu"}}"#, q(source))));
    assert!(diagnostics(&parsed).is_empty(), "parse diagnostics: {parsed}");
    let ir = parsed["ir"].as_str().expect("ir present").to_string();

    let rendered = parse_json(&render_ir_api(&format!(
        r#"{{"ir":{},"context":{}}}"#,
        q(&ir),
        q(r#"{"name":"Ada","ok":true,"items":["a","b"]}"#),
    )));
    assert!(diagnostics(&rendered).is_empty(), "render diagnostics: {rendered}");
    assert_eq!(rendered["output"].as_str(), Some("Hello Ada! ON[a][b]"));
}

#[test]
fn strict_undefined_reports_error_with_variable_arg() {
    let rendered = parse_json(&render_source_api(&format!(
        r#"{{"source":{},"context":{},"options":{}}}"#,
        q("<% missing %>"),
        q("{}"),
        q(r#"{"strict":true}"#),
    )));
    assert!(rendered["output"].is_null());
    let diags = diagnostics(&rendered);
    assert_eq!(diags.len(), 1, "diagnostics: {rendered}");
    assert_eq!(diags[0]["severity"], "error");
    assert_eq!(diags[0]["code"], "dejavu.template.undefined-variable");
    assert!(
        diags[0]["argsJson"].as_str().expect("argsJson string").contains("missing"),
        "argsJson: {}",
        diags[0]["argsJson"]
    );
}

#[test]
fn parse_error_is_structured() {
    let parsed = parse_json(&parse_ir(&format!(r#"{{"source":{}}}"#, q("<% if user")),));
    assert!(parsed["ir"].is_null());
    let diags = diagnostics(&parsed);
    assert_eq!(diags.len(), 1, "diagnostics: {parsed}");
    assert_eq!(diags[0]["severity"], "error");
    assert_eq!(diags[0]["stage"], "parse");
    let code = diags[0]["code"].as_str().expect("code string");
    assert!(code.starts_with("dejavu.parse."), "code: {code}");
    assert!(diags[0]["span"]["start"].is_number(), "span: {parsed}");
    assert!(diags[0]["span"]["end"].is_number(), "span: {parsed}");
}

#[test]
fn invalid_request_json_returns_host_diagnostic() {
    let parsed = parse_json(&parse_ir("{not json"));
    let diags = diagnostics(&parsed);
    assert_eq!(diags.len(), 1, "diagnostics: {parsed}");
    assert_eq!(diags[0]["code"], "dejavu.ir.invalid-json");
    assert_eq!(diags[0]["stage"], "host");
    assert_eq!(diags[0]["severity"], "error");
}

#[test]
fn resolve_template_ref_hits_root_path() {
    let resolved = parse_json(&resolve_template_ref(&format!(
        r#"{{"reference":{},"roots":[{{"name":"main","priority":10,"files":["footer.dejavu"]}}]}}"#,
        q("main:footer.dejavu"),
    )));
    assert!(diagnostics(&resolved).is_empty(), "diagnostics: {resolved}");
    assert_eq!(resolved["id"].as_str(), Some("main:footer.dejavu"));
}

#[test]
fn render_source_with_catalog_include() {
    let footer_parsed = parse_json(&parse_ir(&format!(r#"{{"source":{}}}"#, q(" world!"))));
    let footer_ir = footer_parsed["ir"].as_str().expect("footer ir").to_string();

    let catalog = format!(
        r#"{{"roots":[{{"name":"main","priority":10,"files":{{"footer.dejavu":{}}}}}]}}"#,
        q(&footer_ir),
    );

    let rendered = parse_json(&render_source_api(&format!(
        r#"{{"source":{},"context":{},"catalog":{}}}"#,
        q(r#"hello <% include "footer" %>"#),
        q("{}"),
        q(&catalog),
    )));
    assert!(diagnostics(&rendered).is_empty(), "diagnostics: {rendered}");
    assert_eq!(rendered["output"].as_str(), Some("hello  world!"));
}

#[test]
fn precompile_single_emits_code_and_origins() {
    let parsed = parse_json(&parse_ir(&format!(
        r#"{{"source":{}}}"#,
        q("<% loop item in items %><% item %><% end loop %>"),
    )));
    let ir = parsed["ir"].as_str().expect("ir present").to_string();

    let compiled = parse_json(&precompile_ir_single(&format!(
        r#"{{"ir":{},"options":{}}}"#,
        q(&ir),
        q(r#"{"entryId":"main:page.dejavu"}"#),
    )));
    assert!(diagnostics(&compiled).is_empty(), "diagnostics: {compiled}");
    assert_eq!(compiled["entryId"], "main:page.dejavu");
    let module = &compiled["module"];
    assert_eq!(module["path"], "index.js");
    assert!(!module["code"].as_str().expect("code string").is_empty());
    let origins = module["origins"].as_array().expect("origins array");
    assert!(!origins.is_empty());
    assert!(origins[0]["line"].is_number());
    assert!(origins[0]["start"].is_number());
    assert!(origins[0]["end"].is_number());
}
