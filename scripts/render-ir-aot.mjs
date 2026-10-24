#!/usr/bin/env node
/**
 * Conformance helper: render an IR fixture through AOT codegen + engine runtime.
 * Usage: node scripts/render-ir-aot.mjs <ir.json> <ctx.json>
 *
 * Emits the same `render(ctx, options)` entry as generated projects:
 *   IR JSON -> precompileIrSingle (N-API) -> ESM module -> engine runtime.
 *
 * Requires:
 *   - `pnpm --filter @doki-land/dejavu build` (N-API facade in dist/)
 *   - `pnpm --filter @doki-land/dejavu-engine build` (pure TS runtime)
 *   - a built native addon (`node scripts/build/napi.mjs`)
 *
 * Diagnostics go to stderr. Non-zero exit means AOT render failed.
 */
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const facadeDist = join(root, "projects", "dejavu.ts", "packages", "dejavu", "dist");
const napiEntry = join(facadeDist, "napi.js");
const runtimeEntry = join(
    root,
    "projects",
    "dejavu.ts",
    "packages",
    "dejavu-engine",
    "dist",
    "runtime",
    "index.js",
);

const RUNTIME_IMPORT = "@doki-land/dejavu-engine/runtime";

function fail(message) {
    console.error(`AOT render failed: ${message}`);
    process.exit(1);
}

function describeDiagnostics(diagnostics) {
    return diagnostics.map((d) => `  [${d.severity}] ${d.code}: ${d.message}`).join("\n");
}

if (!existsSync(napiEntry)) {
    fail(
        `missing facade build artifact ${napiEntry}\n` +
            "  build it with: pnpm --filter @doki-land/dejavu build",
    );
}

if (!existsSync(runtimeEntry)) {
    fail(
        `missing dejavu-engine runtime build artifact ${runtimeEntry}\n` +
            "  build it with: pnpm --filter @doki-land/dejavu-engine build",
    );
}

const { precompileIrSingle, resolveNativePath } = await import(pathToFileURL(napiEntry).href);

let nativePath;
try {
    nativePath = resolveNativePath();
} catch (error) {
    fail(
        `cannot resolve native addon: ${error instanceof Error ? error.message : String(error)}\n` +
            "  build it with: node scripts/build/napi.mjs",
    );
}
if (!existsSync(nativePath)) {
    fail(`missing native addon ${nativePath}\n` + "  build it with: node scripts/build/napi.mjs");
}

const irJson = readFileSync(process.argv[2], "utf8");
const ctx = JSON.parse(readFileSync(process.argv[3], "utf8"));

const result = precompileIrSingle(irJson);

if (result.diagnostics.length) {
    console.error(describeDiagnostics(result.diagnostics));
}
if (result.diagnostics.some((d) => d.severity === "error")) {
    process.exit(1);
}

const module = result.modules[0];
if (!module) {
    fail("AOT codegen produced no module");
}

const runtimeUrl = pathToFileURL(runtimeEntry).href;
const rewritten = module.code
    .replace(
        new RegExp(`(\\bfrom\\s*)(["'])${RUNTIME_IMPORT.replace(/[/\\]/g, "\\$&")}\\2`, "g"),
        (_match, prefix, quote) => `${prefix}${quote}${runtimeUrl}${quote}`,
    )
    .replace(
        new RegExp(`(\\bimport\\s*)(["'])${RUNTIME_IMPORT.replace(/[/\\]/g, "\\$&")}\\2`, "g"),
        (_match, prefix, quote) => `${prefix}${quote}${runtimeUrl}${quote}`,
    );

const dataUrl = `data:text/javascript;base64,${Buffer.from(rewritten, "utf8").toString("base64")}`;
const { render } = await import(dataUrl);
process.stdout.write(render(ctx, {}));
