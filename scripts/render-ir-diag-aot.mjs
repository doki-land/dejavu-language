#!/usr/bin/env node
/**
 * Conformance helper: render an IR fixture through AOT codegen + engine runtime
 * and report structured diagnostics.
 * Usage: node scripts/render-ir-diag-aot.mjs <ir.json> <ctx.json> [options.render.json]
 *
 * Requires the same artifacts as `scripts/render-ir-aot.mjs`.
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

if (!existsSync(napiEntry) || !existsSync(runtimeEntry)) {
    console.error("missing facade or engine runtime build artifact");
    process.exit(1);
}

const { precompileIrSingle, resolveNativePath } = await import(pathToFileURL(napiEntry).href);
const nativePath = resolveNativePath();
if (!existsSync(nativePath)) {
    console.error(`missing native addon ${nativePath}`);
    process.exit(1);
}

const irJson = readFileSync(process.argv[2], "utf8");
const ctx = JSON.parse(readFileSync(process.argv[3], "utf8"));
const options = process.argv[4] ? JSON.parse(readFileSync(process.argv[4], "utf8")) : {};

const result = precompileIrSingle(irJson);
if (result.diagnostics.some((d) => d.severity === "error")) {
    process.stdout.write(
        JSON.stringify({
            diagnostics: result.diagnostics.map((d) => ({ code: d.code, stage: d.stage })),
        }),
    );
    process.exit(0);
}

const module = result.modules[0];
const runtimeUrl = pathToFileURL(runtimeEntry).href;
const RUNTIME_IMPORT = "@doki-land/dejavu-engine/runtime";
const rewritten = module.code.replace(
    new RegExp(`(\\bfrom\\s*)(["'])${RUNTIME_IMPORT.replace(/[/\\]/g, "\\$&")}\\2`, "g"),
    (_match, prefix, quote) => `${prefix}${quote}${runtimeUrl}${quote}`,
);

const dataUrl = `data:text/javascript;base64,${Buffer.from(rewritten, "utf8").toString("base64")}`;
const { render } = await import(dataUrl);

try {
    const output = render(ctx, options);
    process.stdout.write(JSON.stringify({ output, diagnostics: [] }));
} catch (error) {
    process.stdout.write(
        JSON.stringify({
            diagnostics: [
                {
                    code: error?.code ?? "dejavu.host.unknown",
                    stage: error?.stage ?? "host",
                },
            ],
        }),
    );
}
