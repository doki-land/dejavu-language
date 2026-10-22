/**
 * Host-side catalog I/O.
 *
 * The engine owns path-v1 semantics (scheme, relative refs, priority, extension
 * probing). This module only walks directories, reads files, parses each template
 * through the engine, and builds the catalog payload the engine consumes.
 */

import { readdirSync, readFileSync } from "node:fs";
import { join, relative, sep } from "node:path";

import { parse } from "./index.js";
import type { DejavuCatalog, DejavuCatalogRoot, IrDocument } from "./types.js";

/** One catalog root backed by a host directory. */
export type CatalogDir = {
    name: string;
    scheme?: string;
    priority?: number;
    dir: string;
    extensions?: string[];
};

const DEFAULT_EXTENSIONS = [".dejavu", ".doki", ".html"];

/** Read every template under `dir` into a parsed-IR catalog root. */
export function loadCatalogFromDirs(
    dirs: CatalogDir[],
    options: { extensions?: string[] } = {},
): DejavuCatalog {
    const extensions = options.extensions ?? DEFAULT_EXTENSIONS;
    const roots: DejavuCatalogRoot[] = dirs.map((entry) => ({
        name: entry.name,
        ...(entry.scheme === undefined ? {} : { scheme: entry.scheme }),
        ...(entry.priority === undefined ? {} : { priority: entry.priority }),
        basePath: entry.dir,
        files: readRoot(entry.dir, entry.extensions ?? extensions),
    }));
    return { roots, extensions: ["", ...extensions] };
}

/** Serialize a catalog for the N-API request (IR values become JSON strings). */
export function catalogToJson(catalog: DejavuCatalog): string {
    const roots = catalog.roots.map((root) => ({
        ...root,
        files: Object.fromEntries(
            Object.entries(root.files).map(([path, ir]) => [
                path,
                typeof ir === "string" ? ir : JSON.stringify(ir),
            ]),
        ),
    }));
    return JSON.stringify({
        extensions: catalog.extensions,
        roots,
    });
}

function readRoot(dir: string, extensions: string[]): Record<string, IrDocument> {
    const files: Record<string, IrDocument> = {};
    for (const file of walk(dir, extensions)) {
        const source = readFileSync(file, "utf8");
        const path = relative(dir, file).split(sep).join("/");
        files[path] = parse(source, { file: path });
    }
    return files;
}

function walk(dir: string, extensions: string[]): string[] {
    const out: string[] = [];
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
        const full = join(dir, entry.name);
        if (entry.isDirectory()) {
            out.push(...walk(full, extensions));
            continue;
        }
        if (entry.isFile() && extensions.some((ext) => entry.name.endsWith(ext))) {
            out.push(full);
        }
    }
    return out.sort();
}
