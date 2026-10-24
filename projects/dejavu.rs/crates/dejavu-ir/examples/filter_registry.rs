//! Dump the language-neutral `filter` / `function` / `test` registries for
//! cross-host conformance.

fn main() {
    println!("{}", dejavu_ir::registry::to_json());
}
