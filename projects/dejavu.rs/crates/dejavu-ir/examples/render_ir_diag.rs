//! CLI helper for conformance: render IR JSON and emit diagnostics as JSON.
//! Usage: render_ir_diag <ir.json> <ctx.json> [options.render.json]
//!
//! Writes `{"diagnostics":[...]}` and exits 0 even when rendering fails, so
//! the conformance runner can compare structured diagnostics across hosts.

use dejavu_ir::registry::CapabilitySet;
use dejavu_ir::{IrError, RenderOptions, render_ir_opts};
use serde_json::{Value, json};
use std::env;
use std::fs;

fn main() {
    let mut args = env::args().skip(1);
    let ir_path = args.next().expect("ir.json path");
    let ctx_path = args.next().expect("ctx.json path");
    let options_path = args.next();

    let ir = fs::read_to_string(ir_path).expect("read ir");
    let ctx: Value =
        serde_json::from_str(&fs::read_to_string(ctx_path).expect("read ctx")).expect("ctx json");

    let mut options = RenderOptions::default();
    if let Some(path) = options_path {
        let raw = fs::read_to_string(path).expect("read options");
        let parsed: Value = serde_json::from_str(&raw).expect("options json");
        options.strict_undefined = parsed["strictUndefined"].as_bool().unwrap_or(false);
        options.name = parsed["name"].as_str().map(str::to_string);
        options.capabilities = match parsed["capabilities"].as_array() {
            Some(names) => {
                let names: Vec<String> =
                    names.iter().map(|name| name.as_str().unwrap_or_default().to_string()).collect();
                CapabilitySet::from_names(&names).expect("known capability")
            }
            None => CapabilitySet::all(),
        };
    }

    let doc: dejavu_ir::IrDocument = serde_json::from_str(&ir).expect("ir json");
    let payload = match render_ir_opts(&doc, &ctx, options, None) {
        Ok(output) => json!({ "output": output, "diagnostics": [] }),
        Err(IrError::Diagnostic(diagnostic)) => json!({ "diagnostics": [diagnostic] }),
        Err(other) => json!({
            "diagnostics": [{
                "severity": "error",
                "code": "dejavu.host.ir-json",
                "stage": "host",
                "message": other.to_string()
            }]
        }),
    };
    println!("{payload}");
}
