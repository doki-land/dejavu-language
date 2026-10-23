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
import { DejavuError } from "./errors.js";
/** File name discovered by `dejavu generate` when `--config` is absent. */
export declare const CONFIG_FILE_NAME = "dejavu.config.ts";
/** Default extension probe order and template admission set. */
export declare const DEFAULT_EXTENSIONS: readonly string[];
/** Default generator-owned output directory, relative to the config file. */
export declare const DEFAULT_OUT_DIR = "src/generated/dejavu";
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
    render: {
        strictUndefined: boolean;
    };
    generate: {
        outDir: string;
        entries: Record<string, string>;
    };
};
/** Options accepted by {@link resolveConfigPath}. */
export type ResolveConfigOptions = {
    /** Explicit `--config` value; relative paths resolve against `cwd`. */
    config?: string;
    /** Discovery start directory. Defaults to `process.cwd()`. */
    cwd?: string;
};
/** Build a structured config-stage error. */
export declare function configError(code: string, message: string, path?: string, args?: Record<string, unknown>): DejavuError;
/**
 * Resolve the config file to load.
 *
 * `--config` wins. Otherwise walk up from `cwd`, checking each directory, and
 * stop at (and including) the nearest directory that has a `package.json` or a
 * `.git` entry so a monorepo package can never pick up a workspace-level
 * config.
 */
export declare function resolveConfigPath(options?: ResolveConfigOptions): string;
/**
 * Load and validate `dejavu.config.ts`.
 *
 * The default export must be a plain object; async factories, functions, and
 * alternate file formats are rejected rather than silently ignored.
 */
export declare function loadConfig(file: string): Promise<ResolvedConfig>;
