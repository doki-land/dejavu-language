/**
 * Authoring surface for `dejavu.config.ts`.
 *
 * This subpath is intentionally dependency-free: it must never import the
 * N-API bridge, the CLI, or `node:fs`, so `import { defineConfig } from
 * "@doki-land/dejavu/config"` stays a pure type-level import.
 *
 * `defineConfig` only narrows the authoring type. Runtime validation happens
 * once in the config loading pipeline (`config-load.ts`) and cannot be
 * bypassed by using this helper.
 */
/** One template root. `name` is both the root name and its reference scheme. */
export type RootConfig = {
    /** Unique root name. Doubles as the scheme in `root:path` references. */
    name: string;
    /** Directory path (no globs). Relative paths resolve against the config file. */
    dir: string;
    /** Higher wins for bare references. Defaults to `0`. */
    priority?: number;
};
/** Render defaults baked into generated entry modules. */
export type RenderConfig = {
    /** Default for generated `render(ctx, options)` calls. Defaults to `false`. */
    strictUndefined?: boolean;
};
/** Generate output selection. */
export type GenerateConfig = {
    /** Generator-owned output directory. Defaults to `./src/generated/dejavu`. */
    outDir?: string;
    /** Public entry name → explicit canonical reference, e.g. `app:pages/home.dejavu`. */
    entries?: Record<string, string>;
};
/** Project configuration accepted by `defineConfig` / `dejavu generate`. */
export type DejavuConfig = {
    /** Template roots. Must be non-empty and uniquely named. */
    roots: RootConfig[];
    /**
     * Extension probe order and template admission set. The empty string means
     * "exact key" and admits suffix-less files during directory scans.
     * Defaults to `["", ".dejavu", ".html"]`.
     */
    extensions?: string[];
    render?: RenderConfig;
    /** Required for `dejavu generate`; omitted for projects that only use the JIT. */
    generate?: GenerateConfig;
};
/**
 * Type helper for `dejavu.config.ts`. Returns the same object, unchanged.
 *
 * ```ts
 * import { defineConfig } from "@doki-land/dejavu/config";
 * export default defineConfig({ roots: [{ name: "app", dir: "./templates" }] });
 * ```
 */
export declare function defineConfig<T extends DejavuConfig>(config: T): T;
