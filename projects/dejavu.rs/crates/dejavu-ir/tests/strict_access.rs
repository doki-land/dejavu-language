//! strict 缺失成员 / 缺失下标语义（Rust oracle）。
//!
//! 值模型不变：缺失标识符 / 成员 / 下标都归一 null。`strict_undefined` 只
//! 把三类缺失访问各报一个独立 code；null 接收者仍宽容；链在第一个缺失环节
//! 报错；默认 lenient 输出零变化。

use dejavu_ir::{ArgValue, IrDocument, IrError, RenderOptions, codes, default_language, render_ir_opts};
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

fn identifier(name: &str) -> serde_json::Value {
    serde_json::json!({ "type": "Expr.Identifier", "name": name })
}

fn member(object: serde_json::Value, property: &str) -> serde_json::Value {
    serde_json::json!({ "type": "Expr.Member", "object": object, "property": property })
}

fn index(object: serde_json::Value, index: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "type": "Expr.Index", "object": object, "index": index })
}

fn literal_num(n: i64) -> serde_json::Value {
    serde_json::json!({ "type": "Expr.Literal", "value": n })
}

fn literal_str(s: &str) -> serde_json::Value {
    serde_json::json!({ "type": "Expr.Literal", "value": s })
}

fn render(expr: serde_json::Value, ctx: Value, strict: bool) -> Result<String, IrError> {
    let doc = tpl_ir(vec![interpolation(expr)]);
    let opts = RenderOptions { strict_undefined: strict, name: Some("t".into()), ..Default::default() };
    render_ir_opts(&doc, &ctx, opts, None)
}

fn ctx_json(map: serde_json::Value) -> Value {
    serde_json::from_value(map).expect("ctx json")
}

fn err_code(err: IrError) -> (String, Vec<(String, String)>) {
    match err {
        IrError::Diagnostic(d) => {
            let mut args: Vec<(String, String)> = d
                .args
                .iter()
                .map(|(k, v)| {
                    let s = match v {
                        ArgValue::String(s) => s.clone(),
                        other => format!("{other:?}"),
                    };
                    (k.clone(), s)
                })
                .collect();
            args.sort();
            (d.code, args)
        }
        other => panic!("expected Diagnostic, got {other:?}"),
    }
}

#[test]
fn strict_missing_member_reports_undefined_member_with_path() {
    let ctx = ctx_json(serde_json::json!({ "user": { "name": "a" } }));
    let (code, args) = err_code(render(member(identifier("user"), "email"), ctx, true).unwrap_err());
    assert_eq!(code, codes::UNDEFINED_MEMBER);
    assert_eq!(args, vec![("name".into(), "email".into()), ("path".into(), "user.email".into()),]);
}

#[test]
fn strict_index_out_of_bounds_reports_undefined_index() {
    let ctx = ctx_json(serde_json::json!({ "items": [1, 2] }));
    let (code, args) = err_code(render(index(identifier("items"), literal_num(5)), ctx, true).unwrap_err());
    assert_eq!(code, codes::UNDEFINED_INDEX);
    assert_eq!(args, vec![("name".into(), "5".into()), ("path".into(), "items[5]".into())]);
}

#[test]
fn strict_object_missing_key_reports_undefined_index() {
    let ctx = ctx_json(serde_json::json!({ "user": { "name": "a" } }));
    let (code, args) = err_code(render(index(identifier("user"), literal_str("email")), ctx, true).unwrap_err());
    assert_eq!(code, codes::UNDEFINED_INDEX);
    assert_eq!(args, vec![("name".into(), "email".into()), ("path".into(), "user[email]".into())]);
}

#[test]
fn strict_chain_reports_at_first_missing_link() {
    // a exists, a.b exists but carries no `c` → error names a.b.c's first gap.
    let ctx = ctx_json(serde_json::json!({ "a": { "b": { "x": 1 } } }));
    let expr = member(member(member(identifier("a"), "b"), "c"), "d");
    let (code, args) = err_code(render(expr, ctx, true).unwrap_err());
    assert_eq!(code, codes::UNDEFINED_MEMBER);
    assert_eq!(args, vec![("name".into(), "c".into()), ("path".into(), "a.b.c".into())]);
}

#[test]
fn strict_null_receiver_stays_lenient() {
    // `a` exists, `a.b` exists and is null → `a.b.c` normalizes to null.
    let ctx = ctx_json(serde_json::json!({ "a": { "b": null } }));
    let expr = member(member(identifier("a"), "b"), "c");
    assert_eq!(render(expr, ctx, true).unwrap(), "");
}

#[test]
fn strict_present_access_renders_normally() {
    let ctx = ctx_json(serde_json::json!({ "items": [7, 8], "user": { "name": "bo" } }));
    assert_eq!(render(index(identifier("items"), literal_num(1)), ctx.clone(), true).unwrap(), "8");
    assert_eq!(render(member(identifier("user"), "name"), ctx, true).unwrap(), "bo");
}

#[test]
fn lenient_output_is_unchanged() {
    let ctx = ctx_json(serde_json::json!({ "items": [1], "user": {} }));
    assert_eq!(render(member(identifier("user"), "email"), ctx.clone(), false).unwrap(), "");
    assert_eq!(render(index(identifier("items"), literal_num(9)), ctx.clone(), false).unwrap(), "");
    assert_eq!(render(index(identifier("user"), literal_str("email")), ctx, false).unwrap(), "");
}

#[test]
fn strict_non_lookup_indexes_stay_lenient() {
    // Negative and fractional indexes never error, even under strict; the
    // pre-existing lenient normalization (falls back to index 0) is unchanged.
    let ctx = ctx_json(serde_json::json!({ "items": [1, 2] }));
    assert_eq!(render(index(identifier("items"), literal_num(-1)), ctx.clone(), true).unwrap(), "1");
    let frac = serde_json::json!({ "type": "Expr.Literal", "value": 1.5 });
    assert_eq!(render(index(identifier("items"), frac), ctx, true).unwrap(), "1");
}

#[test]
fn strict_dynamic_index_path_uses_question_label() {
    // `items[i]` where `i` is an identifier that exists → path label `?`.
    let ctx = ctx_json(serde_json::json!({ "items": [1], "i": 4 }));
    let (code, args) = err_code(render(index(identifier("items"), identifier("i")), ctx, true).unwrap_err());
    assert_eq!(code, codes::UNDEFINED_INDEX);
    assert_eq!(args, vec![("name".into(), "4".into()), ("path".into(), "items[?]".into())]);
}
