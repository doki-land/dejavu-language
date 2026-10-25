//! CLI helper for conformance: render IR JSON and emit diagnostics as JSON.
//! Usage: render_ir_diag <ir.json> <ctx.json> [--options <options.render.json>]
//! [--meta <meta.json>]
//!
//! `meta.json#customRegistry` supplies host registry descriptions. This helper
//! has no implementations: fixtures that reach dispatch report
//! `dejavu.template.custom-invocation`, so validation-only fixtures are the
//! only supported shape here.
//!
//! Writes `{"diagnostics":[...]}` and exits 0 even when rendering fails, so
//! the conformance runner can compare structured diagnostics across hosts.

use dejavu_ir::registry::{CapabilitySet, CustomEntry};
use dejavu_ir::{CustomError, CustomRegistry, IrError, RenderOptions, render_ir_opts};
use serde_json::{Value, json};
use std::env;
use std::fs;

/// Registry descriptions without implementations.
struct DescribedRegistry {
    entries: Vec<CustomEntry>,
}

impl CustomRegistry for DescribedRegistry {
    fn entries(&self) -> &[CustomEntry] {
        &self.entries
    }

    fn apply_filter(&self, name: &str, _value: &Value, _args: &[Value]) -> Result<Value, CustomError> {
        Err(CustomError::new(format!("fixture registry has no `{name}` implementation")))
    }

    fn call_function(&self, name: &str, _args: &[Value]) -> Result<Value, CustomError> {
        Err(CustomError::new(format!("fixture registry has no `{name}` implementation")))
    }

    fn apply_test(&self, name: &str, _value: &Value, _args: &[Value]) -> Result<bool, CustomError> {
        Err(CustomError::new(format!("fixture registry has no `{name}` implementation")))
    }
}

fn main() {
    let mut ir_path = None;
    let mut ctx_path = None;
    let mut options_path = None;
    let mut meta_path = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--options" => options_path = args.next(),
            "--meta" => meta_path = args.next(),
            _ if ir_path.is_none() => ir_path = Some(arg),
            _ if ctx_path.is_none() => ctx_path = Some(arg),
            other => panic!("unexpected argument `{other}`"),
        }
    }
    let ir_path = ir_path.expect("ir.json path");
    let ctx_path = ctx_path.expect("ctx.json path");

    let ir = fs::read_to_string(ir_path).expect("read ir");
    let ctx: Value =
        serde_json::from_str(&fs::read_to_string(ctx_path).expect("read ctx")).expect("ctx json");

    let registry = meta_path.map(|path| {
        let meta: Value =
            serde_json::from_str(&fs::read_to_string(path).expect("read meta")).expect("meta json");
        let entries: Vec<CustomEntry> = serde_json::from_value(meta["customRegistry"].clone())
            .expect("customRegistry entries");
        DescribedRegistry { entries }
    });

    let mut options = RenderOptions::default();
    options.custom = registry.as_ref().map(|registry| registry as &dyn CustomRegistry);
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
