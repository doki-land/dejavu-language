#!/usr/bin/env node
/**
 * Conformance helper: render an IR fixture through the WASM language bridge.
 * Usage: node scripts/render-ir-wasm.mjs <ir.json> <ctx.json>
 *
 * A `meta.json#customRegistry` / `registry.mjs` next to the IR document becomes
 * the host registry for the render.
 *
 * Requires `node scripts/build/wasm.mjs` so the platform package carries a
 * wasm build of the current Rust sources.
 */
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { loadFixtureRegistry } from "./registry-fixture.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const { initWasm, renderIr } = await import(
    pathToFileURL(join(root, "projects", "dejavu.ts", "packages", "dejavu-language", "dist", "index.js")).href
);
const wasmPath = join(
    root,
    "projects",
    "dejavu.ts",
    "platforms",
    "dejavu-unknown-wasm32",
    "src",
    "lib",
    "dejavu_wasm_bg.wasm",
);

await initWasm({ module: readFileSync(wasmPath) });

const irJson = readFileSync(process.argv[2], "utf8");
const ctx = JSON.parse(readFileSync(process.argv[3], "utf8"));
const registry = await loadFixtureRegistry(dirname(resolve(process.argv[2])));
const result = renderIr({ ir: irJson, context: ctx, registry });
if (result.diagnostics.length > 0) {
    console.error(JSON.stringify(result.diagnostics, null, 2));
    process.exit(1);
}
process.stdout.write(result.output ?? "");
