/**
 * Config discovery, loading, and strict validation for `dejavu generate`.
 *
 * The config file is project-owned TypeScript evaluated by Node itself, so the
 * package only needs Node's native type stripping (`>=22.18`) and no extra
 * parser dependency. Every path in the result is resolved against the config
 * file's directory — never the process cwd.
 *
 * Errors are `DejavuError` instances with `stage: "config"`; nothing here
 * touches the N-API bridge, the CLI, or template sources.
 */

import { realpathSync, statSync } from "node:fs";
import { basename, dirname, isAbsolute, join, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";

import { DejavuError } from "./errors.js";

/** File name discovered by `dejavu generate` when `--config` is absent. */
export const CONFIG_FILE_NAME = "dejavu.config.ts";

/** Default extension probe order and template admission set. */
export const DEFAULT_EXTENSIONS: readonly string[] = ["", ".dejavu", ".html"];

/** Default generator-owned output directory, relative to the config file. */
export const DEFAULT_OUT_DIR = "src/generated/dejavu";

/** Normalized, path-resolved config consumed by the generate pipeline. */
export type ResolvedRoot = {
    name: string;
    /** Absolute directory path. */
    dir: string;
    priority: number;
};

/** Normalized config with every directory resolved against the config file. */
export type ResolvedConfig = {
    /** Absolute path to the loaded config file. */
    file: string;
    /** Absolute directory containing the config file. */
    dir: string;
    roots: ResolvedRoot[];
    extensions: string[];
    render: { strictUndefined: boolean };
    generate: { outDir: string; entries: Record<string, string> };
};

/** Options accepted by {@link resolveConfigPath}. */
export type ResolveConfigOptions = {
    /** Explicit `--config` value; relative paths resolve against `cwd`. */
    config?: string;
    /** Discovery start directory. Defaults to `process.cwd()`. */
    cwd?: string;
};

/** Build a structured config-stage error. */
export function configError(
    code: string,
    message: string,
    path?: string,
    args: Record<string, unknown> = {},
): DejavuError {
    return new DejavuError({
        code,
        severity: "error",
        stage: "config",
        message,
        ...(path ? { path } : {}),
        argsJson: JSON.stringify(args),
        related: [],
    });
}

/**
 * Resolve the config file to load.
 *
 * `--config` wins. Otherwise walk up from `cwd`, checking each directory, and
 * stop at (and including) the nearest directory that has a `package.json` or a
 * `.git` entry so a monorepo package can never pick up a workspace-level
 * config.
 */
export function resolveConfigPath(options: ResolveConfigOptions = {}): string {
    const cwd = resolve(options.cwd ?? process.cwd());

    if (typeof options.config === "string" && options.config.trim() !== "") {
        const explicit = isAbsolute(options.config)
            ? resolve(options.config)
            : resolve(cwd, options.config);
        if (!isFile(explicit)) {
            throw configError(
                "dejavu.config.not-found",
                `config file not found: ${explicit}`,
                explicit,
                { config: options.config },
            );
        }
        return explicit;
    }

    const searched: string[] = [];
    let dir = cwd;
    for (;;) {
        searched.push(dir);
        const candidate = join(dir, CONFIG_FILE_NAME);
        if (isFile(candidate)) return candidate;
        if (isFile(join(dir, "package.json")) || isDirectory(join(dir, ".git"))) break;
        const parent = dirname(dir);
        if (parent === dir || parent === "") break;
        dir = parent;
    }

    throw configError(
        "dejavu.config.not-found",
        `no ${CONFIG_FILE_NAME} found from ${cwd} up to the nearest package root. ` +
            "Pass --config <file> to select one explicitly.",
        cwd,
        { searched: searched.join(",") },
    );
}

/** Cache-busting counter so `--watch` always evaluates a fresh config module. */
let loadSequence = 0;

/**
 * Load and validate `dejavu.config.ts`.
 *
 * The default export must be a plain object; async factories, functions, and
 * alternate file formats are rejected rather than silently ignored.
 */
export async function loadConfig(file: string): Promise<ResolvedConfig> {
    const abs = resolve(file);
    if (!isFile(abs)) {
        throw configError("dejavu.config.not-found", `config file not found: ${abs}`, abs);
    }
    const configDir = dirname(abs);

    let module: { default?: unknown };
    try {
        loadSequence += 1;
        module = (await import(`${pathToFileURL(abs).href}?dejavu=${loadSequence}`)) as {
            default?: unknown;
        };
    } catch (error) {
        throw configError(
            "dejavu.config.load-failed",
            `failed to load ${abs}: ${errorText(error)}`,
            abs,
        );
    }

    const value = module.default;
    if (value === undefined) {
        throw configError(
            "dejavu.config.missing-default",
            `${abs} must default-export the object created by defineConfig()`,
            abs,
        );
    }
    if (value instanceof Promise) {
        throw configError(
            "dejavu.config.async-not-supported",
            `${abs} defaults to a Promise. Async config factories are not supported.`,
            abs,
        );
    }

    const config = asRecord(value);
    if (!config) {
        throw configError(
            "dejavu.config.invalid-shape",
            `${abs} must default-export a plain object`,
            abs,
        );
    }

    rejectUnknownFields(config, ["roots", "extensions", "render", "generate"], "config", abs);

    const roots = parseRoots(config.roots, configDir, abs);
    const extensions = parseExtensions(config.extensions, abs);
    const render = parseRender(config.render, abs);

    if (config.generate === undefined) {
        throw configError(
            "dejavu.config.missing-generate",
            `${abs} must declare a "generate" block with at least one entry`,
            abs,
        );
    }
    const generate = parseGenerate(config.generate, configDir, roots, abs);

    return {
        file: abs,
        dir: configDir,
        roots,
        extensions,
        render,
        generate,
    };
}

// ---------------------------------------------------------------------------
// Field validation
// ---------------------------------------------------------------------------

function parseRoots(value: unknown, configDir: string, file: string): ResolvedRoot[] {
    if (!Array.isArray(value) || value.length === 0) {
        throw configError(
            "dejavu.config.invalid-roots",
            `"roots" must be a non-empty array`,
            file,
        );
    }

    const out: ResolvedRoot[] = [];
    const names = new Set<string>();
    for (const [index, raw] of value.entries()) {
        const root = asRecord(raw);
        if (!root) {
            throw configError(
                "dejavu.config.invalid-root",
                `roots[${index}] must be an object with "name" and "dir"`,
                file,
                { index },
            );
        }
        rejectUnknownFields(root, ["name", "dir", "priority"], `roots[${index}]`, file);

        const name = root.name;
        if (typeof name !== "string" || name.trim() === "") {
            throw configError(
                "dejavu.config.invalid-root",
                `roots[${index}].name must be a non-empty string`,
                file,
                { index },
            );
        }
        if (/[:\\/\s]/.test(name)) {
            throw configError(
                "dejavu.config.invalid-root-name",
                `root name ${JSON.stringify(name)} must not contain ":", "/", "\\", or whitespace`,
                file,
                { name },
            );
        }
        if (names.has(name)) {
            throw configError(
                "dejavu.config.duplicate-root",
                `duplicate root name ${JSON.stringify(name)}`,
                file,
                { name },
            );
        }
        names.add(name);

        const dir = root.dir;
        if (typeof dir !== "string" || dir.trim() === "") {
            throw configError(
                "dejavu.config.invalid-root",
                `roots[${index}].dir must be a non-empty directory path`,
                file,
                { name },
            );
        }
        const absDir = resolve(configDir, dir);
        if (!isDirectory(absDir)) {
            throw configError(
                "dejavu.config.root-missing",
                `root ${JSON.stringify(name)} directory not found: ${absDir}`,
                absDir,
                { name, dir },
            );
        }

        const priority = root.priority;
        if (priority !== undefined && (typeof priority !== "number" || !Number.isInteger(priority))) {
            throw configError(
                "dejavu.config.invalid-root",
                `roots[${index}].priority must be an integer`,
                file,
                { name },
            );
        }

        out.push({ name, dir: absDir, priority: priority ?? 0 });
    }

    return out;
}

function parseExtensions(value: unknown, file: string): string[] {
    if (value === undefined) return [...DEFAULT_EXTENSIONS];
    if (!Array.isArray(value) || value.length === 0) {
        throw configError(
            "dejavu.config.invalid-extensions",
            `"extensions" must be a non-empty array such as ["", ".dejavu"]`,
            file,
        );
    }

    const out: string[] = [];
    const seen = new Set<string>();
    for (const [index, ext] of value.entries()) {
        if (typeof ext !== "string") {
            throw configError(
                "dejavu.config.invalid-extensions",
                `extensions[${index}] must be a string`,
                file,
                { index },
            );
        }
        if (ext !== "" && !ext.startsWith(".")) {
            throw configError(
                "dejavu.config.invalid-extensions",
                `extensions[${index}] must be empty or start with "." (got ${JSON.stringify(ext)})`,
                file,
                { index, extension: ext },
            );
        }
        if (seen.has(ext)) {
            throw configError(
                "dejavu.config.invalid-extensions",
                `duplicate extension ${JSON.stringify(ext)}`,
                file,
                { extension: ext },
            );
        }
        seen.add(ext);
        out.push(ext);
    }
    return out;
}

function parseRender(value: unknown, file: string): { strictUndefined: boolean } {
    if (value === undefined) return { strictUndefined: false };
    const render = asRecord(value);
    if (!render) {
        throw configError("dejavu.config.invalid-render", `"render" must be an object`, file);
    }
    rejectUnknownFields(render, ["strictUndefined"], "render", file);
    if (render.strictUndefined !== undefined && typeof render.strictUndefined !== "boolean") {
        throw configError(
            "dejavu.config.invalid-render",
            `"render.strictUndefined" must be a boolean`,
            file,
        );
    }
    return { strictUndefined: render.strictUndefined === true };
}

function parseGenerate(
    value: unknown,
    configDir: string,
    roots: ResolvedRoot[],
    file: string,
): { outDir: string; entries: Record<string, string> } {
    const generate = asRecord(value);
    if (!generate) {
        throw configError("dejavu.config.invalid-generate", `"generate" must be an object`, file);
    }
    rejectUnknownFields(generate, ["outDir", "entries"], "generate", file);

    const outDirRaw = generate.outDir;
    if (outDirRaw !== undefined && (typeof outDirRaw !== "string" || outDirRaw.trim() === "")) {
        throw configError(
            "dejavu.config.invalid-out-dir",
            `"generate.outDir" must be a non-empty directory path`,
            file,
        );
    }
    const outDir = resolve(configDir, outDirRaw ?? DEFAULT_OUT_DIR);
    assertOutputDir(outDir, configDir, roots, file);

    const entriesRaw = generate.entries;
    const entries = asRecord(entriesRaw);
    if (!entries || Object.keys(entries).length === 0) {
        throw configError(
            "dejavu.config.invalid-entries",
            `"generate.entries" must be a non-empty object mapping public names to canonical references`,
            file,
        );
    }

    const rootNames = new Set(roots.map((root) => root.name));
    const publicNames = new Map<string, string>();
    const parsed: Record<string, string> = {};

    for (const [name, reference] of Object.entries(entries)) {
        assertPublicEntryName(name, file);
        const folded = name.normalize("NFKC").toLowerCase();
        const clash = publicNames.get(folded);
        if (clash !== undefined) {
            throw configError(
                "dejavu.config.entry-name-collision",
                `entry names ${JSON.stringify(clash)} and ${JSON.stringify(name)} collide after case folding`,
                file,
                { name, other: clash },
            );
        }
        publicNames.set(folded, name);

        if (typeof reference !== "string" || reference.trim() === "") {
            throw configError(
                "dejavu.config.invalid-entry",
                `entry ${JSON.stringify(name)} must map to a canonical reference such as "app:welcome.dejavu"`,
                file,
                { name },
            );
        }
        const colon = reference.indexOf(":");
        if (colon <= 0 || colon === reference.length - 1) {
            throw configError(
                "dejavu.config.invalid-entry",
                `entry ${JSON.stringify(name)} must use an explicit "root:path" reference (got ${JSON.stringify(reference)})`,
                file,
                { name, reference },
            );
        }
        const rootName = reference.slice(0, colon);
        const templatePath = reference.slice(colon + 1);
        if (!rootNames.has(rootName)) {
            throw configError(
                "dejavu.config.invalid-entry",
                `entry ${JSON.stringify(name)} references unknown root ${JSON.stringify(rootName)}`,
                file,
                { name, root: rootName },
            );
        }
        if (templatePath.includes("\\") || templatePath.startsWith("/") || hasDotSegments(templatePath)) {
            throw configError(
                "dejavu.config.invalid-entry",
                `entry ${JSON.stringify(name)} must use a POSIX relative path without "." or ".." segments`,
                file,
                { name, reference },
            );
        }
        parsed[name] = reference;
    }

    return { outDir, entries: parsed };
}

const RESERVED_ENTRY_NAMES = new Set([
    "break", "case", "catch", "class", "const", "continue", "debugger", "default",
    "delete", "do", "else", "enum", "export", "extends", "false", "finally", "for",
    "function", "if", "import", "in", "instanceof", "new", "null", "return", "super",
    "switch", "this", "throw", "true", "try", "typeof", "var", "void", "while", "with",
    "yield", "await", "implements", "interface", "let", "package", "private", "protected",
    "public", "static", "index", "_internal", "manifest",
]);

function assertPublicEntryName(name: string, file: string): void {
    if (!/^[A-Za-z_$][A-Za-z0-9_$]*$/.test(name)) {
        throw configError(
            "dejavu.config.invalid-entry-name",
            `entry name ${JSON.stringify(name)} must be a plain JavaScript identifier without path separators`,
            file,
            { name },
        );
    }
    if (RESERVED_ENTRY_NAMES.has(name)) {
        throw configError(
            "dejavu.config.invalid-entry-name",
            `entry name ${JSON.stringify(name)} is reserved (JS keywords, "index", "_internal", "manifest" are rejected)`,
            file,
            { name },
        );
    }
}

/** Reject output directories that overlap the config directory or any input root. */
function assertOutputDir(
    outDir: string,
    configDir: string,
    roots: ResolvedRoot[],
    file: string,
): void {
    const out = canonicalPath(outDir);
    const configRoot = canonicalPath(configDir);

    if (sameOrAncestor(out, configRoot)) {
        throw configError(
            "dejavu.config.out-dir-conflict",
            `output directory ${outDir} must be a subdirectory of the project, not the project or workspace root`,
            file,
            { outDir },
        );
    }

    for (const root of roots) {
        const rootDir = canonicalPath(root.dir);
        if (sameOrAncestor(out, rootDir) || sameOrAncestor(rootDir, out)) {
            throw configError(
                "dejavu.config.out-dir-conflict",
                `output directory ${outDir} must not overlap template root ${JSON.stringify(root.name)} (${root.dir})`,
                file,
                { outDir, root: root.name },
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Filesystem / path helpers
// ---------------------------------------------------------------------------

function isFile(path: string): boolean {
    try {
        return statSync(path).isFile();
    } catch {
        return false;
    }
}

function isDirectory(path: string): boolean {
    try {
        return statSync(path).isDirectory();
    } catch {
        return false;
    }
}

function hasDotSegments(path: string): boolean {
    return path.split("/").some((segment) => segment === "." || segment === ".." || segment === "");
}

function errorText(error: unknown): string {
    return error instanceof Error ? error.message : String(error);
}

/**
 * Resolve symlinks for the existing prefix and re-append the missing tail, so
 * overlap checks compare real locations even before `outDir` exists.
 */
function canonicalPath(path: string): string {
    const absolute = resolve(path);
    let current = absolute;
    const tail: string[] = [];
    for (;;) {
        try {
            const real = realpathSync(current);
            const joined = tail.reverse().reduce((acc, part) => join(acc, part), real);
            return normalizeCase(joined);
        } catch {
            const parent = dirname(current);
            if (parent === current) return normalizeCase(absolute);
            tail.push(basename(current));
            current = parent;
        }
    }
}

function normalizeCase(path: string): string {
    return process.platform === "win32" ? path.toLowerCase() : path;
}

function sameOrAncestor(candidate: string, other: string): boolean {
    return candidate === other || other.startsWith(candidate + sep);
}

// ---------------------------------------------------------------------------
// Shape helpers
// ---------------------------------------------------------------------------

function asRecord(value: unknown): Record<string, unknown> | undefined {
    if (!value || typeof value !== "object" || Array.isArray(value)) return undefined;
    return value as Record<string, unknown>;
}

function rejectUnknownFields(
    value: Record<string, unknown>,
    allowed: string[],
    where: string,
    file: string,
): void {
    const allowedSet = new Set(allowed);
    for (const key of Object.keys(value)) {
        if (allowedSet.has(key)) continue;
        throw configError(
            "dejavu.config.unknown-field",
            `unknown field ${JSON.stringify(key)} in ${where} (allowed: ${allowed.join(", ")})`,
            file,
            { field: key, where },
        );
    }
}
