//! `irVersion` contract gate: only `"v0"` deserializes; unknown, missing, or
//! non-string versions are rejected explicitly (no fallback parsing).

use dejavu_ir::{IR_VERSION, IrDocument, decode_ir_json, default_language};
use serde_json::{Value, json};

fn doc_json(ir_version: Value) -> Value {
    json!({
        "irVersion": ir_version,
        "language": default_language(),
        "body": { "type": "Template", "children": [] }
    })
}

#[test]
fn accepts_v0() {
    assert_eq!(IR_VERSION, "v0");
    let doc: IrDocument = serde_json::from_value(doc_json(json!("v0"))).expect("v0 must deserialize");
    assert_eq!(doc.ir_version, IR_VERSION);

    let raw = serde_json::to_string(&doc_json(json!("v0"))).expect("serialize");
    decode_ir_json(&raw).expect("decode_ir_json must accept v0");
}

#[test]
fn rejects_unknown_versions() {
    for bad in ["1.0", "0", "v1", "V0", "", "v0.1", "v0 "] {
        let err = serde_json::from_value::<IrDocument>(doc_json(json!(bad)))
            .expect_err(&format!("irVersion {bad:?} must be rejected"));
        assert!(err.to_string().contains("irVersion"), "rejection must name the field: {err}");
    }
    serde_json::from_value::<IrDocument>(doc_json(json!(1.0)))
        .expect_err("non-string irVersion must be rejected");
    serde_json::from_value::<IrDocument>(doc_json(Value::Null))
        .expect_err("null irVersion must be rejected");
}

#[test]
fn rejects_missing_version() {
    let mut doc = doc_json(json!("v0"));
    doc.as_object_mut().expect("object").remove("irVersion");
    serde_json::from_value::<IrDocument>(doc).expect_err("missing irVersion must be rejected");
}
