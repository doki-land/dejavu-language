#!/usr/bin/env node
/**
 * Conformance helper: render an IR fixture through the desktop facade and
 * report structured diagnostics.
 * Usage: node scripts/render-ir-diag-ts.mjs <ir.json> <ctx.json> [options.render.json]
 */
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const facadeDist = join(root, "projects", "dejavu.ts", "packages", "dejavu", "dist", "index.js");
if (!existsSync(facadeDist)) {
    console.error(`missing facade build artifact ${facadeDist}`);
    process.exit(1);
}

const { renderIr, DejavuError } = await import(pathToFileURL(facadeDist).href);

const irJson = readFileSync(process.argv[2], "utf8");
const ctx = JSON.parse(readFileSync(process.argv[3], "utf8"));
const options = process.argv[4] ? JSON.parse(readFileSync(process.argv[4], "utf8")) : {};

try {
    const output = renderIr(irJson, ctx, options);
    process.stdout.write(JSON.stringify({ output, diagnostics: [] }));
} catch (error) {
    if (error instanceof DejavuError) {
        process.stdout.write(
            JSON.stringify({
                diagnostics: [{ code: error.code, stage: error.stage, span: error.span, args: error.args }],
            }),
        );
    } else {
        process.stdout.write(
            JSON.stringify({
                diagnostics: [
                    {
                        code: "dejavu.host.unknown",
                        stage: "host",
                        message: error instanceof Error ? error.message : String(error),
                    },
                ],
            }),
        );
    }
}
