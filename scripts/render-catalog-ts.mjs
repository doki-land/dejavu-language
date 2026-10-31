#!/usr/bin/env node
/**
 * Conformance helper: render a multi-template inheritance fixture through
 * the `@doki-land/dejavu` facade (N-API → Rust parser + renderer).
 *
 * Usage: node scripts/render-catalog-ts.mjs <fixture_dir> [diagnostics]
 *
 * Reads `entry.txt`, `templates/*`, `context.ctx.json` and optional
 * `language.json` from the fixture dir, builds a catalog via
 * `loadCatalogFromDirs`, then renders the entry template source through
 * `renderSource`. Passes `delimiters` through to support non-default
 * `<% %>` code delimiters (e.g. Jinja-style `{% %}`).
 *
 * With the `diagnostics` mode the script emits `{"diagnostics":[...]}` (code /
 * stage / args) instead of the render output and exits 0, matching the Rust
 * `catalog_diag` example shape.
 */
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const { parseIr, renderSource } = await import(
    pathToFileURL(join(root, "projects", "dejavu.ts", "packages", "dejavu", "dist", "napi.js")).href
);

const fixtureDir = process.argv[2];
const mode = process.argv[3];
const templatesDir = join(fixtureDir, "templates");
const ctx = JSON.parse(readFileSync(join(fixtureDir, "context.ctx.json"), "utf8"));
const entryName = readFileSync(join(fixtureDir, "entry.txt"), "utf8").trim();

// ---- optional language.json ----
let delimiters = undefined;
const languagePath = join(fixtureDir, "language.json");
if (existsSync(languagePath)) {
    delimiters = readFileSync(languagePath, "utf8").trim();
}

// ---- find entry template source ----
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

// ---- parse all templates into IR (with configured delimiters) ----
const extensions = [".dejavu", ".doki", ".jinja", ".html", ".tmpl", ".tpl"];
const rootEntries = [];
for (const entry of readdirSync(templatesDir, { withFileTypes: true })) {
    if (!entry.isFile()) continue;
    const name = entry.name;
    if (!extensions.some((ext) => name.endsWith(ext))) continue;
    const source = readFileSync(join(templatesDir, name), "utf8");
    const parsed = parseIr(source, { file: name, ...(delimiters ? { delimiters } : {}) });
    const errors = parsed.diagnostics.filter((d) => d.severity === "error");
    if (errors.length > 0 || !parsed.irJson) {
        // In diagnostics mode a parse failure is itself the fixture's
        // expected cross-host diagnostic (stage `parse`).
        if (mode === "diagnostics") {
            const first = errors[0];
            const diagnostic = first
                ? { code: first.code, stage: first.stage, args: JSON.parse(first.argsJson || "{}") }
                : { code: "dejavu.host.unknown", stage: "host", args: {} };
            process.stdout.write(JSON.stringify({ diagnostics: [diagnostic] }));
            process.exit(0);
        }
        console.error(`failed to parse ${name}: ${errors.map((d) => d.message).join("; ")}`);
        process.exit(2);
    }
    // The IR body's Template.children is what the catalog stores
    rootEntries.push([name, parsed.irJson]);
}

// ---- build catalog manually ----
const catalog = {
    extensions: ["", ...extensions],
    roots: [
        {
            name: "fixture",
            scheme: "fixture",
            priority: 10,
            basePath: templatesDir,
            files: Object.fromEntries(rootEntries),
        },
    ],
};

const render = () => renderSource(entrySource, ctx, { catalog: JSON.stringify(catalog), delimiters });

if (mode === "diagnostics") {
    try {
        render();
        process.stdout.write(JSON.stringify({ diagnostics: [] }));
    } catch (error) {
        // Only structured engine diagnostics participate. Anything else is a
        // host failure and must not look like a cycle diagnostic.
        if (!error || typeof error.code !== "string") throw error;
        const diagnostic = { code: error.code, stage: error.stage, args: error.args ?? {} };
        process.stdout.write(JSON.stringify({ diagnostics: [diagnostic] }));
    }
} else {
    process.stdout.write(render());
}
