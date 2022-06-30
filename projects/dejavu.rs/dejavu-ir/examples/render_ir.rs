//! CLI helper for conformance: render IR JSON with context JSON.
//! Usage: render_ir <ir.json> <ctx.json>

use dejavu_ir::render_ir_json;
use std::env;
use std::fs;

fn main() {
    let mut args = env::args().skip(1);
    let ir_path = args.next().expect("ir.json path");
    let ctx_path = args.next().expect("ctx.json path");
    let ir = fs::read_to_string(ir_path).expect("read ir");
    let ctx: serde_json::Value = serde_json::from_str(&fs::read_to_string(ctx_path).expect("read ctx")).expect("ctx json");
    let out = render_ir_json(&ir, &ctx).expect("render");
    print!("{out}");
}
