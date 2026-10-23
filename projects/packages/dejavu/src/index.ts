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

import { DejavuError } from "./errors.js";
import {
    parseIr as nativeParseIr,
    normalizeIr as nativeNormalizeIr,
    renderIr as nativeRenderIr,
    renderSource as nativeRenderSource,
    precompileIrSingle as nativePrecompileIrSingle,
    resolveTemplateRef as nativeResolveTemplateRef,
    bindingContractVersion as nativeBindingContractVersion,
} from "./napi.js";
import {
    Template,
} from "./types.js";
import type {
    Catalog,
    CompileOptions,
    Diagnostic,
    ParseOptions,
    PrecompileCatalogResult,
    PrecompileModule,
    PrecompileOptions,
    PrecompileResult,
    RenderOptions,
    ResolveTemplateRefRequest,
    ResolveTemplateRefResult,
    TemplateContext,
} from "./types.js";

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

export { DejavuError } from "./errors.js";
export { Template };
export type {
    Catalog,
    CatalogRoot,
    CompileOptions,
    Diagnostic,
    PrecompileCatalogResult,
    PrecompileModule,
    PrecompileOptions,
    PrecompileResult,
    ParseOptions,
    RelatedSpan,
    RenderOptions,
    ResolveTemplateRefRequest,
    ResolveTemplateRefResult,
    Severity,
    Span,
    TemplateContext,
} from "./types.js";

// ---------------------------------------------------------------------------
// Low-level catalog wire helper (kept local — not public API yet)
// ---------------------------------------------------------------------------

function catalogToJson(catalog: Catalog): string {
    const roots = catalog.roots.map((root) => ({
        ...root,
        files: Object.fromEntries(
            Object.entries(root.files).map(([path, source]) => {
                // Serialize each source through parseIr → IR JSON.
                const parsed = nativeParseIr(source, { file: path });
                if (!parsed.irJson) {
                    // If parse fails at catalog-build time, surface the first error.
                    throwIfErrors(parsed.diagnostics);
                    // Shouldn't reach here — throwIfErrors returns on error.
                }
                return [path, parsed.irJson!];
            }),
        ),
    }));
    return JSON.stringify({
        extensions: catalog.extensions,
        roots,
    });
}

// ---------------------------------------------------------------------------
// Author-facing API
// ---------------------------------------------------------------------------

/**
 * Single-shot JIT render. Parse `source` once, render against `ctx`, return
 * the output string. Throws `DejavuError` on error-severity diagnostics.
 *
 * This is the shortest path from template source + context → output.
 */
export function render(
    source: string,
    ctx: TemplateContext = {},
    options: RenderOptions = {},
): string {
    const result = nativeRenderSource(source, ctx, {
        ...options,
        ...(options.name ? {} : { name: "<main>" }),
    });
    return result;
}

/**
 * JIT-prepare `source` into a reusable `Template`. The source is parsed
 * exactly once; each call to `template.render(ctx)` re-uses the prepared
 * IR without re-parsing.
 *
 * `Template` is process-local, non-serializable, and NOT safe to pass
 * across Worker boundaries.
 */
export function compile(
    source: string,
    options: CompileOptions = {},
): Template {
    const parseResult = nativeParseIr(source, {
        file: options.file ?? "<template>",
        delimiters: options.delimiters,
    });
    throwIfErrors(parseResult.diagnostics);
    if (!parseResult.irJson) {
        throw new DejavuError({
            code: "dejavu.compile.no-ir",
            severity: "error",
            stage: "compile",
            message: "compile(): parseIr returned no IR and no error diagnostic",
            argsJson: "{}",
            related: [],
        });
    }

    return new Template(
        parseResult.irJson,
        options,
        (irJson, ctx, renderOpts) => {
            return nativeRenderIr(irJson, ctx, renderOpts);
        },
    );
}

/**
 * AOT-compile a single template to ESM JavaScript. Returns the full
 * artifact (modules + diagnostics + entryId) in memory — does NOT write
 * files. Use CLI (`dejavu compile <file> --out <file.js>`) or combine with
 * FS code yourself to persist artifacts.
 *
 * The generated ESM module imports `@doki-land/dejavu-engine/runtime` (pure TS,
 * no N-API dependency).
 */
export function precompile(
    source: string,
    options: PrecompileOptions & { file?: string } = {},
): PrecompileResult {
    const parseResult = nativeParseIr(source, {
        file: options.file ?? options.entryId ?? "__entry__",
    });
    throwIfErrors(parseResult.diagnostics);
    if (!parseResult.irJson) {
        throw new DejavuError({
            code: "dejavu.precompile.no-ir",
            severity: "error",
            stage: "compile",
            message: "precompile(): parseIr returned no IR and no error diagnostic",
            argsJson: "{}",
            related: [],
        });
    }
    const result = nativePrecompileIrSingle(parseResult.irJson, {
        strictUndefined: options.strictUndefined,
        entryId: options.entryId,
    });
    return result;
}

// ---------------------------------------------------------------------------
// Advanced API
// ---------------------------------------------------------------------------

/**
 * Parse template source into an IR JSON object. Throws `DejavuError` on
 * error-severity diagnostics.
 */
export function parse(
    source: string,
    options: ParseOptions = {},
): unknown {
    const result = nativeParseIr(source, options);
    throwIfErrors(result.diagnostics);
    if (!result.irJson) {
        throw new DejavuError({
            code: "dejavu.parse.no-ir",
            severity: "error",
            stage: "parse",
            message: "parse(): native returned no IR and no error diagnostic",
            argsJson: "{}",
            related: [],
        });
    }
    return JSON.parse(result.irJson);
}

/**
 * Normalize an IR document into its canonical JSON form (strips spans,
 * empty text nodes, etc). Throws `DejavuError` on errors.
 */
export function normalizeIr(irJson: string): unknown {
    const result = nativeNormalizeIr(irJson);
    throwIfErrors(result.diagnostics);
    if (!result.irJson) {
        throw new DejavuError({
            code: "dejavu.normalize.no-ir",
            severity: "error",
            stage: "host",
            message: "normalizeIr(): native returned no IR and no error diagnostic",
            argsJson: "{}",
            related: [],
        });
    }
    return JSON.parse(result.irJson);
}

/**
 * Render an IR JSON string (advanced).
 */
export function renderIr(
    irJson: string,
    ctx: TemplateContext = {},
    options: RenderOptions = {},
): string {
    return nativeRenderIr(irJson, ctx, options);
}

/**
 * Resolve a path-v1 template reference without touching the filesystem.
 */
export function resolveTemplateRef(
    request: ResolveTemplateRefRequest,
): ResolveTemplateRefResult {
    return nativeResolveTemplateRef(request);
}

/** Binding contract reported by the loaded native addon. */
export function bindingContractVersion(): string {
    return nativeBindingContractVersion();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function throwIfErrors(diagnostics: Diagnostic[]): void {
    const error = DejavuError.fromDiagnostics(diagnostics);
    if (error) throw error;
}
