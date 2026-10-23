#!/usr/bin/env node
/**
 * Conformance helper: render an IR fixture through the `@doki-land/dejavu` facade.
 * Usage: node scripts/render-ir-ts.mjs <ir.json> <ctx.json>
 *
 * Requires `pnpm --filter @doki-land/dejavu build` (the facade loads the N-API addon).
 */
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const { renderIr } = await import(
    pathToFileURL(join(root, "projects", "packages", "dejavu", "dist", "index.js")).href
);

// Read IR JSON as string directly — facade `renderIr` takes IR JSON string.
const irJson = readFileSync(process.argv[2], "utf8");
const ctx = JSON.parse(readFileSync(process.argv[3], "utf8"));
process.stdout.write(renderIr(irJson, ctx));
