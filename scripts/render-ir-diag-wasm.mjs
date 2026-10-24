#!/usr/bin/env node
/**
 * Conformance helper: render an IR fixture through the WASM bridge and report
 * structured diagnostics.
 * Usage: node scripts/render-ir-diag-wasm.mjs <ir.json> <ctx.json> [options.render.json]
 *
 * Requires `node scripts/build/wasm.mjs`.
 */
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

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
const options = process.argv[4] ? JSON.parse(readFileSync(process.argv[4], "utf8")) : {};

const result = renderIr({ ir: irJson, context: ctx, options });
process.stdout.write(
    JSON.stringify({
        output: result.output,
        diagnostics: result.diagnostics.map((d) => ({ code: d.code, stage: d.stage, span: d.span })),
    }),
);
