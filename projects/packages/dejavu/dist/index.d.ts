/**
 * Author-facing facade over the `dejavu-napi` bridge.
 *
 * This module exposes the small core surface documented in
 * `dejavu-typescript-ux.md §3`:
 *   - `render(source, context, options?)`   — single-shot JIT
 *   - `compile(source, options?)`           — JIT prep, returns reusable Template
 *   - `precompile(source, options?)`        — AOT, returns memory artifacts
 *   - `parse` / `normalizeIr` / `renderIr` / `resolveTemplateRef` — advanced
 *   - `DejavuError` and all public types
 *
 * Every function here is a thin, typed composition of one or more capabilities
 * from `./napi.js`. Command semantics (argv, config, stdio, exit codes) belong
 * to `./cli.js`.
 */
import { Template } from "./types.js";
import type { Catalog, CompileOptions, ParseOptions, PrecompileCatalogResult, PrecompileOptions, PrecompileResult, RenderOptions, ResolveTemplateRefRequest, ResolveTemplateRefResult, TemplateContext } from "./types.js";
export { DejavuError } from "./errors.js";
export { Template };
export type { Catalog, CatalogRoot, CompileOptions, Diagnostic, PrecompileCatalogResult, PrecompileModule, PrecompileOptions, PrecompileResult, ParseOptions, RelatedSpan, RenderOptions, ResolveTemplateRefRequest, ResolveTemplateRefResult, Severity, Span, TemplateContext, } from "./types.js";
/**
 * Single-shot JIT render. Parse `source` once, render against `ctx`, return
 * the output string. Throws `DejavuError` on error-severity diagnostics.
 *
 * This is the shortest path from template source + context → output.
 */
export declare function render(source: string, ctx?: TemplateContext, options?: RenderOptions): string;
/**
 * JIT-prepare `source` into a reusable `Template`. The source is parsed
 * exactly once; each call to `template.render(ctx)` re-uses the prepared
 * IR without re-parsing.
 *
 * `Template` is process-local, non-serializable, and NOT safe to pass
 * across Worker boundaries.
 */
export declare function compile(source: string, options?: CompileOptions): Template;
/**
 * AOT-compile a single template to ESM JavaScript. Returns the full
 * artifact (modules + diagnostics + entryId) in memory — does NOT write
 * files. Use CLI (`dejavu compile <file> --out <file.js>`) or combine with
 * FS code yourself to persist artifacts.
 *
 * The generated ESM module imports `@doki-land/dejavu-engine/runtime` (pure TS,
 * no N-API dependency).
 */
export declare function precompile(source: string, options?: PrecompileOptions & {
    file?: string;
}): PrecompileResult;
/**
 * AOT-compile a multi-template catalog.
 *
 * `options.entryId` selects the canonical entry id (e.g. `app:pages/home.dejavu`).
 * The entry module is emitted as `index.js`; every template reachable through
 * `include` / `extends` is emitted as `_internal/<sanitized-id>.js`. Artifacts
 * stay in memory — `dejavu generate` (see `generate.ts`) owns the on-disk
 * layout, declarations, source maps, and manifest.
 */
export declare function precompileCatalog(catalog: Catalog, options?: PrecompileOptions): PrecompileCatalogResult;
/**
 * Parse template source into an IR JSON object. Throws `DejavuError` on
 * error-severity diagnostics.
 */
export declare function parse(source: string, options?: ParseOptions): unknown;
/**
 * Normalize an IR document into its canonical JSON form (strips spans,
 * empty text nodes, etc). Throws `DejavuError` on errors.
 */
export declare function normalizeIr(irJson: string): unknown;
/**
 * Render an IR JSON string (advanced).
 */
export declare function renderIr(irJson: string, ctx?: TemplateContext, options?: RenderOptions): string;
/**
 * Resolve a path-v1 template reference without touching the filesystem.
 */
export declare function resolveTemplateRef(request: ResolveTemplateRefRequest): ResolveTemplateRefResult;
/** Binding contract reported by the loaded native addon. */
export declare function bindingContractVersion(): string;
