use dejavu_ir::{decode_ir_json, encode_native, normalize_ir_json, render_ir_json};
use dejavu_language::parse;
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn conformance_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../specifications/conformance/t1")
}

#[test]
fn t1_render_from_expected_ir() {
    let root = conformance_root();
    assert!(root.is_dir(), "missing {}", root.display());

    let mut cases: Vec<_> = fs::read_dir(&root).unwrap().filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).collect();
    cases.sort_by_key(|e| e.file_name());

    for entry in cases {
        let dir = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let expected_ir = fs::read_to_string(dir.join("expected.ir.json")).unwrap();
        let ctx: Value = serde_json::from_str(&fs::read_to_string(dir.join("context.ctx.json")).unwrap()).unwrap();
        let expected_out = fs::read_to_string(dir.join("expected.out.txt")).unwrap();

        let out = render_ir_json(&expected_ir, &ctx).unwrap_or_else(|e| panic!("{name}: render {e}"));
        assert_eq!(out, expected_out, "render mismatch in case `{name}`");
    }
}

#[test]
fn native_roundtrip_hello() {
    let src = "Hello, <% name %>!";
    let doc = parse(src).unwrap();
    let json = serde_json::to_string_pretty(&doc).unwrap();
    let native = decode_ir_json(&json).unwrap();
    let again = encode_native(&native);
    let a = normalize_ir_json(&serde_json::to_string(&doc).unwrap()).unwrap();
    let b = normalize_ir_json(&serde_json::to_string(&again).unwrap()).unwrap();
    assert_eq!(a, b);
}
