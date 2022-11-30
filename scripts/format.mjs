#!/usr/bin/env node
/**
 * Format the whole repository: Rust (`cargo fmt`) + JS/TS/JSON (Biome).
 *
 *   node scripts/format.mjs           # write
 *   node scripts/format.mjs --check   # check only
 */
import {execFileSync} from "node:child_process";
import {existsSync, readdirSync} from "node:fs";
import {createRequire} from "node:module";
import {dirname, join} from "node:path";
import {fileURLToPath} from "node:url";

const rootDir = join(dirname(fileURLToPath(import.meta.url)), "..");
const checkOnly = process.argv.includes("--check");

/**
 * @param {string} file
 * @param {string[]} args
 * @param {string} [cwd]
 */
function runFile(file, args, cwd = rootDir) {
    console.log(`$ ${file} ${args.join(" ")}`);
    execFileSync(file, args, {cwd, stdio: "inherit", env: process.env});
}

function rustManifests() {
    const projects = join(rootDir, "projects");
    if (!existsSync(projects)) {
        return [];
    }
    return readdirSync(projects, {withFileTypes: true})
        .filter((d) => d.isDirectory())
        .map((d) => join("projects", d.name, "Cargo.toml"))
        .filter((rel) => existsSync(join(rootDir, rel)));
}

/**
 * Workspace package names only (excludes external path deps).
 * @param {string} manifestRel
 * @returns {string[]}
 */
function workspacePackageNames(manifestRel) {
    const raw = execFileSync(
        "cargo",
        ["metadata", "--no-deps", "--format-version", "1", `--manifest-path=${manifestRel}`],
        {cwd: rootDir, encoding: "utf8", env: process.env},
    );
    const meta = JSON.parse(raw);
    const members = new Set(meta.workspace_members);
    return meta.packages
        .filter((pkg) => members.has(pkg.id))
        .map((pkg) => pkg.name)
        .sort();
}

function resolveBiome() {
    const require = createRequire(join(rootDir, "package.json"));
    try {
        return require.resolve("@biomejs/biome/bin/biome");
    } catch {
        return null;
    }
}

console.log(checkOnly ? "=== Format check ===\n" : "=== Format (write) ===\n");

const manifests = rustManifests();
if (manifests.length > 0) {
    console.log("--- Rust ---");
    for (const manifest of manifests) {
        const packages = workspacePackageNames(manifest);
        if (packages.length === 0) {
            console.log(`(skip ${manifest}: no workspace packages)`);
            continue;
        }
        const args = [
            "fmt",
            `--manifest-path=${manifest}`,
            ...packages.flatMap((name) => ["-p", name]),
            ...(checkOnly ? ["--check"] : []),
        ];
        runFile("cargo", args);
    }
} else {
    console.log("--- Rust --- (skip: no Cargo.toml under projects/)");
}

const biome = resolveBiome();
if (biome) {
    console.log("\n--- Biome ---");
    // Biome 2.x: `format --check` was removed; use `check` without `--write`.
    const args = checkOnly
        ? ["check", "--linter-enabled=false", "--assist-enabled=false", "."]
        : ["format", "--write", "."];
    runFile(process.execPath, [biome, ...args]);
} else {
    console.log("\n--- Biome --- (skip: @biomejs/biome not installed; run pnpm install)");
}

console.log(checkOnly ? "\n=== Format check passed ===" : "\n=== Format complete ===");
