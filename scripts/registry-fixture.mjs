/**
 * Load a custom-registry fixture next to an IR document.
 *
 * `meta.json#customRegistry` carries the language-neutral entry descriptions
 * (the same wire AOT codegen consumes). `registry.mjs`, when present, exports
 * one implementation table per `filter` / `function` / `test` namespace. A
 * diagnostics fixture may ship descriptions only: validation rejects the
 * render before anything would be dispatched.
 */
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

export async function loadFixtureRegistry(dir) {
    const metaPath = join(dir, "meta.json");
    const entries = existsSync(metaPath)
        ? JSON.parse(readFileSync(metaPath, "utf8")).customRegistry ?? []
        : [];
    const registryPath = join(dir, "registry.mjs");
    const module = existsSync(registryPath) ? await import(pathToFileURL(registryPath).href) : undefined;
    if (!entries.length && !module) return undefined;
    return {
        entries,
        filter: module?.filter,
        function: module?.["function"],
        test: module?.test,
    };
}
