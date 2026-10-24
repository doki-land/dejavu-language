/**
 * Build the `dejavu-wasm` crate for `wasm32-unknown-unknown` and place the
 * generated bindings in the platform package at
 * `projects/dejavu.ts/platforms/dejavu-unknown-wasm32/src/lib`.
 *
 * The platform package is the published carrier for the bridge. The Rust crate
 * stays the single source of truth, so `src/lib` is generated and git-ignored.
 *
 * Usage: `node scripts/build/wasm.mjs [--release]`
 */
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const crateDir = join(root, "projects", "dejavu.rs", "crates", "dejavu-wasm");
const outDir = join(root, "projects", "dejavu.ts", "platforms", "dejavu-unknown-wasm32", "src", "lib");
const release = process.argv.includes("--release");

if (!existsSync(join(crateDir, "Cargo.toml"))) {
    console.error(`missing crate manifest: ${join(crateDir, "Cargo.toml")}`);
    process.exit(1);
}

mkdirSync(dirname(outDir), { recursive: true });
rmSync(outDir, { recursive: true, force: true });

const args = [
    "build",
    crateDir,
    "--target",
    "web",
    "--no-pack",
    "--no-opt",
    "--out-name",
    "dejavu_wasm",
    "--out-dir",
    outDir,
];
if (release) args.push("--release");

const result = spawnSync("wasm-pack", args, { stdio: "inherit" });
if (result.status !== 0) {
    console.error("wasm-pack build failed");
    process.exit(result.status ?? 1);
}

for (const file of ["dejavu_wasm.js", "dejavu_wasm_bg.wasm", "dejavu_wasm.d.ts"]) {
    if (!existsSync(join(outDir, file))) {
        console.error(`missing build output: ${join(outDir, file)}`);
        process.exit(1);
    }
}

console.log(`dejavu-wasm → ${outDir}`);
