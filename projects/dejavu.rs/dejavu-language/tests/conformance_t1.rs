use dejavu_ir::{normalize_ir_json, render_ir_json};
use dejavu_language::parse;
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn conformance_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../specifications/conformance/t1")
}

#[test]
fn t1_cases_parse_and_render() {
    let root = conformance_root();
    assert!(root.is_dir(), "missing {}", root.display());

    let mut cases: Vec<_> = fs::read_dir(&root).unwrap().filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).collect();
    cases.sort_by_key(|e| e.file_name());

    for entry in cases {
        let dir = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let input = fs::read_to_string(dir.join("input.dejavu")).unwrap();
        let expected_ir = fs::read_to_string(dir.join("expected.ir.json")).unwrap();
        let ctx: Value = serde_json::from_str(&fs::read_to_string(dir.join("context.ctx.json")).unwrap()).unwrap();
        let expected_out = fs::read_to_string(dir.join("expected.out.txt")).unwrap();

        let doc = parse(&input).unwrap_or_else(|e| panic!("{name}: parse {e:?}"));
        let got_ir = serde_json::to_string(&doc).unwrap();
        let norm_got = normalize_ir_json(&got_ir).unwrap();
        let norm_exp = normalize_ir_json(&expected_ir).unwrap();
        assert_eq!(norm_got, norm_exp, "IR mismatch in case `{name}`");

        let out = render_ir_json(&expected_ir, &ctx).unwrap_or_else(|e| panic!("{name}: render {e}"));
        assert_eq!(out, expected_out, "render mismatch in case `{name}`");
    }
}

#[test]
fn miette_highlights_unclosed_code() {
    let err = parse("Hello <% name").expect_err("should fail");
    let msg = format!("{err:?}");
    assert!(msg.contains("unclosed") || msg.contains("code"), "expected diagnostic about unclosed code, got {msg}");
}
