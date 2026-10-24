//! Dump the language-neutral filter registry for cross-host conformance.

fn main() {
    println!("{}", dejavu_ir::registry::to_json());
}
