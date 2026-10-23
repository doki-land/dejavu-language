//! Single-template AOT codegen entry point.
//!
//! Produces one ESM module per template IR. No catalog traversal happens here
//! — `precompileCatalog` is intentionally deferred to a future step.

use dejavu_diagnostic::Diagnostic;
use dejavu_ir::{IrDocument, IrNode};
use serde_json::Value;

use crate::emit::{CodegenCtx, emit_body};
use crate::options::CodegenOptions;

/// Result of a single-template precompile.
#[derive(Debug, Clone)]
pub struct PrecompileResult {
    /// Complete ESM module source, ready to `import { render } from "...";`.
    pub code: String,
    /// Imported template dependencies (includes / extends). Always empty for
    /// the single-template codegen path — catalog resolution fills this later.
    pub dependencies: Vec<String>,
    /// Structured diagnostics collected during compilation (Compile stage).
    pub diagnostics: Vec<Diagnostic>,
    /// Entry template id used for dependency tracking. Defaults to `"__entry__"`.
    pub entry_id: String,
}

/// Import path for the runtime helpers.
const RUNTIME_IMPORT: &str = "@doki-land/dejavu-engine/runtime";

/// Top-level entry point.
///
/// - `ir_json` — IR document JSON (as emitted by dejavu-ir / dejavu-language).
/// - `options_json` — optional JSON string with codegen knobs.
pub fn precompile_single(
    ir_json: &str,
    options_json: Option<&str>,
) -> Result<PrecompileResult, String> {
    let doc: IrDocument = serde_json::from_str(ir_json)
        .map_err(|e| format!("invalid IR JSON: {e}"))?;

    let opts = CodegenOptions::from_json(options_json)
        .map_err(|e| format!("invalid options JSON: {e}"))?;

    let entry_id = opts.entry_id().to_string();
    let mut ctx = CodegenCtx::new(&opts);

    let body_str = match &doc.body {
        IrNode::Template { children, .. } => {
            let mut out = String::new();
            emit_body(children, &mut ctx, &mut out);
            out
        }
        other => {
            ctx.diagnostic(
                dejavu_diagnostic::Severity::Error,
                "dejavu.compile.root-not-template",
                format!("IR root node must be Template, got {other:?}"),
            );
            String::new()
        }
    };

    let code = build_module(&body_str);

    Ok(PrecompileResult {
        code,
        dependencies: Vec::new(),
        diagnostics: ctx.diagnostics,
        entry_id,
    })
}

/// Wrap the raw function body fragments into a full ESM module.
fn build_module(body: &str) -> String {
    let mut s = String::with_capacity(512 + body.len());
    s.push_str(&format!(
        "import {{ htmlEscape, applyFilter, isTruthy, binary, unary, valueToString, exprMember, exprIndex }} from \"{RUNTIME_IMPORT}\";\n\n"
    ));
    s.push_str("export function render(ctx, options = {}) {\n");
    s.push_str("  if (!ctx || typeof ctx !== 'object') ctx = {};\n");
    s.push_str("  const __strict = options.strictUndefined === true;\n");
    s.push_str("  const __out = [];\n");
    s.push('\n');
    // Indent each line of the body by 2 spaces for readability.
    for line in body.lines() {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            s.push('\n');
        } else {
            s.push_str("  ");
            s.push_str(trimmed);
            s.push('\n');
        }
    }
    s.push_str("  return __out.join(\"\");\n");
    s.push_str("}\n");
    s
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse IR JSON to an IrDocument (public for tests / future crates).
pub fn parse_ir_json(json: &str) -> Result<IrDocument, String> {
    serde_json::from_str(json).map_err(|e| format!("invalid IR JSON: {e}"))
}

/// Parse options JSON to a Value (public for callers who want extra knobs later).
pub fn parse_options_json(json: Option<&str>) -> Option<Value> {
    json.and_then(|j| serde_json::from_str::<Value>(j).ok())
}
