/**
 * Project-level AOT pipeline behind `dejavu generate`.
 *
 * Responsibilities, in execution order:
 *   1. snapshot template roots into a `Catalog` (`buildCatalog`)
 *   2. AOT-compile every configured entry against that catalog
 *   3. map the in-memory modules to a deterministic disk layout
 *      (`planOutput`) — declarations, barrel, source maps, manifest
 *   4. publish (`writeOutput`) or compare without writing (`checkOutput`)
 *
 * The output directory is owned by this generator: files are only replaced or
 * removed when the previous manifest records them and their content hash is
 * unchanged. Directories are never cleared recursively.
 */
import type { ResolvedConfig } from "./config-load.js";
import type { Catalog, Diagnostic, PrecompileModule } from "./types.js";
/** One file to publish, keyed by its POSIX path relative to the output dir. */
export type GeneratedFile = {
    path: string;
    content: string;
};
/** Source snapshot recorded by the manifest. */
export type ManifestSource = {
    /** Canonical template id (`root:path`). */
    id: string;
    /** `sha256:<hex>` of the template source. */
    hash: string;
};
/** Content-addressed list of generated files (manifest.json excluded). */
export type ManifestFile = {
    path: string;
    hash: string;
};
/** `manifest.json` payload. Contains no absolute paths or volatile values. */
export type GenerateManifest = {
    contract: string;
    runtime: {
        import: string;
        fingerprint: string;
    };
    codegen: {
        contract: string;
        fingerprint: string;
    };
    config: {
        fingerprint: string;
        extensions: string[];
        strictUndefined: boolean;
        roots: Array<{
            name: string;
            dir: string;
            priority: number;
        }>;
    };
    entries: Record<string, string>;
    sources: ManifestSource[];
    files: ManifestFile[];
};
/** Full, in-memory generation result. */
export type GeneratePlan = {
    config: ResolvedConfig;
    /** Absolute output directory. */
    outDir: string;
    /** Files to publish, sorted by path. `manifest.json` is appended at publish time. */
    files: GeneratedFile[];
    manifest: GenerateManifest;
    /** Serialized `manifest.json` including its trailing newline. */
    manifestSource: string;
    /** Non-error diagnostics collected while compiling entries. */
    warnings: Diagnostic[];
};
/** One compiled entry. */
export type CompiledEntry = {
    name: string;
    canonicalId: string;
    modules: PrecompileModule[];
    /** Rust catalog manifest (canonical id → module path). */
    manifestJson: string;
};
/** Aggregated compile output accepted by {@link planOutput}. */
export type GenerateCompilation = {
    catalog: Catalog;
    entries: CompiledEntry[];
    /** Non-error diagnostics collected while compiling entries. */
    warnings: Diagnostic[];
};
/** Options shared by the generate entry points. */
export type BuildGenerateOptions = {
    /** Explicit `--config` value. */
    config?: string;
    /** Directory used for config discovery. Defaults to `process.cwd()`. */
    cwd?: string;
};
/** Drift categories reported by {@link checkOutput}. */
export type DriftKind = "missing" | "changed" | "stale" | "unowned-file" | "invalid-manifest" | "not-a-directory";
/** One mismatch between the planned output and the contents on disk. */
export type Drift = {
    path: string;
    kind: DriftKind;
};
/** Discover config, snapshot templates, compile every entry, and plan output. */
export declare function buildGeneratePlan(options?: BuildGenerateOptions): Promise<GeneratePlan>;
/**
 * Snapshot every configured root into a catalog.
 *
 * Paths are canonicalized to POSIX and sorted, so the resulting catalog (and
 * every artifact derived from it) is byte-stable for the same inputs.
 */
export declare function buildCatalog(config: ResolvedConfig): Catalog;
/** Map compiled modules onto the public output layout. */
export declare function planOutput(config: ResolvedConfig, compilation: GenerateCompilation): GeneratePlan;
/**
 * Publish a plan.
 *
 * Ownership rules: an existing non-empty directory without a valid manifest is
 * refused, recorded files whose content no longer matches the recorded hash are
 * refused, and unrecorded files are never deleted. Stale files are removed one
 * by one — the directory itself is never cleared.
 */
export declare function writeOutput(plan: GeneratePlan, outDir?: string): void;
/**
 * Compare a plan with the output on disk without writing anything.
 *
 * Missing, changed, stale, unrecorded, and malformed-manifest states are all
 * reported as drift; an empty directory therefore never passes a check for a
 * non-empty plan.
 */
export declare function checkOutput(plan: GeneratePlan, outDir?: string): Drift[];
/** Output sink used by watch mode. */
export type WatchIO = {
    stdout(text: string): void;
    stderr(text: string): void;
};
/**
 * Run a full generate, then rebuild on template/config changes.
 *
 * A failed rebuild keeps the previous successful output untouched and keeps
 * watching. The returned promise never settles; termination is the host's
 * interrupt handling.
 */
export declare function watchGenerate(options?: BuildGenerateOptions & {
    io?: WatchIO;
}): Promise<number>;
/** Print a generate failure through the CLI-style diagnostic format. */
export declare function reportGenerateError(error: unknown, io: WatchIO): void;
