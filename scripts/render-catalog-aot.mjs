#!/usr/bin/env node
/**
 * Conformance helper: render a multi-template fixture through AOT codegen.
 *
 * Usage: node scripts/render-catalog-aot.mjs <fixture_dir>
 *
 * Parses every `templates/*` file (honouring optional `language.json`), builds
 * the catalog wire format, runs `precompileCatalog`, materialises the modules
 * under `.conformance-tmp/aot-catalog/<case>/`, then imports the entry module
 * and renders its `render(ctx, options)` export.
 */
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";
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

const { parseIr, precompileCatalog, resolveNativePath } = await import(
    pathToFileURL(napiEntry).href
);
const nativePath = resolveNativePath();
if (!existsSync(nativePath)) {
    console.error(`missing native addon ${nativePath}`);
    process.exit(1);
}

const fixtureDir = process.argv[2];
const templatesDir = join(fixtureDir, "templates");
const ctx = JSON.parse(readFileSync(join(fixtureDir, "context.ctx.json"), "utf8"));
const entryName = readFileSync(join(fixtureDir, "entry.txt"), "utf8").trim();

let delimiters;
const languagePath = join(fixtureDir, "language.json");
if (existsSync(languagePath)) {
    delimiters = readFileSync(languagePath, "utf8").trim();
}

const extensions = [".dejavu", ".doki", ".jinja", ".html", ".tmpl", ".tpl"];
const files = {};
for (const entry of readdirSync(templatesDir, { withFileTypes: true })) {
    if (!entry.isFile()) continue;
    if (!extensions.some((ext) => entry.name.endsWith(ext))) continue;
    const source = readFileSync(join(templatesDir, entry.name), "utf8");
    const parsed = parseIr(source, { file: entry.name, ...(delimiters ? { delimiters } : {}) });
    const errors = parsed.diagnostics.filter((d) => d.severity === "error");
    if (errors.length > 0 || !parsed.irJson) {
        console.error(`failed to parse ${entry.name}: ${errors.map((d) => d.message).join("; ")}`);
        process.exit(2);
    }
    files[entry.name] = parsed.irJson;
}

const catalog = {
    extensions: ["", ...extensions],
    roots: [
        {
            name: "fixture",
            scheme: "fixture",
            priority: 10,
            basePath: templatesDir,
            files,
        },
    ],
};

const entryId = `fixture:${entryName}`;
const result = precompileCatalog("", JSON.stringify(catalog), { entryId });
const errors = result.diagnostics.filter((d) => d.severity === "error");
if (errors.length > 0) {
    console.error(errors.map((d) => `  [error] ${d.code}: ${d.message}`).join("\n"));
    process.exit(2);
}

// Materialise generated modules so their relative imports resolve.
const outDir = join(root, ".conformance-tmp", "aot-catalog", basename(fixtureDir));
rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });
writeFileSync(join(outDir, "package.json"), JSON.stringify({ type: "module" }));

const runtimeUrl = pathToFileURL(runtimeEntry).href;
const runtimeImport = "@doki-land/dejavu-engine/runtime";
for (const module of result.modules) {
    const target = join(outDir, module.path);
    mkdirSync(dirname(target), { recursive: true });
    // Rewrite the package specifier to a file URL so the temp copy runs unmodified.
    const code = module.code.replaceAll(
        `"${runtimeImport}"`,
        JSON.stringify(runtimeUrl),
    ).replaceAll(
        `'${runtimeImport}'`,
        JSON.stringify(runtimeUrl),
    );
    writeFileSync(target, code);
}

const entryModule = await import(pathToFileURL(join(outDir, "index.js")).href);
process.stdout.write(entryModule.render(ctx, {}));
