#!/usr/bin/env node
/**
 * Conformance helper: render an IR fixture through the `@doki-land/dejavu` facade.
 * Usage: node scripts/render-ir-ts.mjs <ir.json> <ctx.json>
 *
 * Requires `pnpm --filter @doki-land/dejavu build` (the facade loads the N-API addon).
 */
import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { loadFixtureRegistry } from "./registry-fixture.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const { renderIr } = await import(
    pathToFileURL(join(root, "projects", "dejavu.ts", "packages", "dejavu", "dist", "index.js")).href
);

// Read IR JSON as string directly — facade `renderIr` takes IR JSON string.
const irJson = readFileSync(process.argv[2], "utf8");
const ctx = JSON.parse(readFileSync(process.argv[3], "utf8"));
const registry = await loadFixtureRegistry(dirname(resolve(process.argv[2])));
process.stdout.write(renderIr(irJson, ctx, registry ? { custom: registry } : {}));
