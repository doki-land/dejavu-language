#!/usr/bin/env node
/**
 * Conformance helper: render a multi-template inheritance fixture through
 * the `@doki-land/dejavu` facade (N-API → Rust parser + renderer).
 *
 * Usage: node scripts/render-catalog-ts.mjs <fixture_dir>
 *
 * Reads `entry.txt`, `templates/*`, `context.ctx.json` from the fixture
 * dir, builds a catalog via `loadCatalogFromDirs`, then renders the entry
 * template source through `renderSource`.
 */
import { readFileSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const { loadCatalogFromDirs, renderSource } = await import(
    pathToFileURL(join(root, "runtimes", "dejavu", "dist", "index.js")).href
);

const fixtureDir = process.argv[2];
const templatesDir = join(fixtureDir, "templates");
const ctx = JSON.parse(readFileSync(join(fixtureDir, "context.ctx.json"), "utf8"));
const entryName = readFileSync(join(fixtureDir, "entry.txt"), "utf8").trim();

// --- build catalog from template dir ---
const catalog = loadCatalogFromDirs([{ name: "fixture", dir: templatesDir }]);

// --- find entry template source ---
let entrySource = "";
for (const entry of readdirSync(templatesDir, { withFileTypes: true })) {
    if (!entry.isFile()) continue;
    if (entry.name === entryName) {
        entrySource = readFileSync(join(templatesDir, entry.name), "utf8");
        break;
    }
}

if (!entrySource) {
    console.error(`entry template "${entryName}" not found in ${templatesDir}`);
    process.exit(2);
}

const out = renderSource(entrySource, ctx, { catalog });
process.stdout.write(out);
