/**
 * N-API bridge over `dejavu-napi`.
 *
 * This module owns:
 *  1. Loading the platform-specific native addon (env override → optional
 *     dependency package resolution).
 *  2. Translating N-API snake_case wire shapes into author-facing camelCase
 *     types from `./types.js`.
 *
 * Command semantics (argv, config, exit codes) belong to `./cli.js`.
 * High-level author-facing API belongs to `./index.js`.
 */

import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { DejavuError } from "./errors.js";
import type {
    Diagnostic,
    PrecompileCatalogResult,
    PrecompileModule,
    PrecompileOptions,
    PrecompileResult,
    RenderOptions,
    Severity,
    Span,
    TemplateContext,
    TemplateContext as _TemplateContext,
} from "./types.js";

// Re-export alias for convenience.
export type { Diagnostic, PrecompileResult, PrecompileCatalogResult };

// ---------------------------------------------------------------------------
// Native engine wire type (snake_case, exactly as N-API returns).
// ---------------------------------------------------------------------------

/** Low-level native exports; MUST match `dejavu-napi/src/lib.rs`. */
type NativeEngine = {
    bindingContractVersion(): string;
    parseIr(request: { source: string; file?: string; language?: string; delimiters?: string }): {
        ir?: string;
        diagnostics: NativeDiagnostic[];
    };
    normalizeIr(request: { ir: string }): { ir?: string; diagnostics: NativeDiagnostic[] };
    renderIr(request: {
        ir: string;
        context: string;
        options?: string;
        catalog?: string;
    }): { output?: string; diagnostics: NativeDiagnostic[] };
    renderSource(request: {
        source: string;
        context: string;
        options?: string;
        catalog?: string;
        delimiters?: string;
    }): { output?: string; diagnostics: NativeDiagnostic[] };
    resolveTemplateRef(request: {
        reference: string;
        from?: string;
        extensions?: string[];
        roots?: Array<{
            name: string;
            scheme?: string;
            priority?: number;
            files: string[];
        }>;
    }): { id?: string; diagnostics: NativeDiagnostic[] };
    precompileIrSingle(request: { ir: string; options?: string }): {
        module?: NativeGeneratedModule;
        diagnostics: NativeDiagnostic[];
        entryId: string;
    };
    precompileCatalog(request: {
        entryIr: string;
        catalog: string;
        options?: string;
    }): {
        modules: NativeGeneratedModule[];
        manifest?: string;
        diagnostics: NativeDiagnostic[];
        entryId: string;
    };
};

type NativeSpan = { start: number; end: number };
type NativeRelatedSpan = { start: number; end: number; label: string; path?: string | null };
type NativeDiagnostic = {
    code: string;
    severity: string;
    stage: string;
    message: string;
    path?: string | null;
    templateId?: string | null;
    span?: NativeSpan | null;
    argsJson: string;
    related: NativeRelatedSpan[];
};
type NativeGeneratedModule = {
    path: string;
    code: string;
    dependencies: string[];
    origins: NativeGeneratedOrigin[];
};
type NativeGeneratedOrigin = {
    line: number;
    start: number;
    end: number;
};

// ---------------------------------------------------------------------------
// Addon loading
// ---------------------------------------------------------------------------

/** Binding contract version this facade is written against. */
export const BINDING_CONTRACT = "dejavu.napi/2";

const PLATFORM_PACKAGES: Record<string, string> = {
    "linux-x64": "@doki-land/dejavu-linux-x64",
    "linux-arm64": "@doki-land/dejavu-linux-arm64",
    "darwin-x64": "@doki-land/dejavu-darwin-x64",
    "darwin-arm64": "@doki-land/dejavu-darwin-arm64",
    "win32-x64": "@doki-land/dejavu-win32-x64",
    "win32-arm64": "@doki-land/dejavu-win32-arm64",
};

let cached: NativeEngine | undefined;

/** Resolve the addon path without loading it (useful for diagnostics). */
export function resolveNativePath(): string {
    const override = process.env.DEJAVU_NATIVE_NODE;
    if (override) {
        return isAbsolute(override) ? override : resolve(override);
    }

    const key = `${process.platform}-${process.arch}`;
    const pkg = PLATFORM_PACKAGES[key];
    if (!pkg) {
        throw new Error(
            `Dejavu native addon does not support platform ${key}. ` +
                "Set DEJAVU_NATIVE_NODE to a built addon or add a platform package.",
        );
    }

    const require = createRequire(import.meta.url);
    let pkgJson: string;
    try {
        pkgJson = require.resolve(`${pkg}/package.json`);
    } catch {
        throw new Error(
            `Dejavu platform package ${pkg} is not installed (platform ${key}). ` +
                "Reinstall @doki-land/dejavu with optional dependencies enabled.",
        );
    }

    const pkgDir = dirname(pkgJson);
    const meta = require(pkgJson) as { main?: string };
    return join(pkgDir, meta.main ?? "index.node");
}

/** Load (and cache) the native addon. Throws if the addon cannot be found. */
export function loadNative(): NativeEngine {
    if (cached) return cached;

    const file = resolveNativePath();
    if (!existsSync(file)) {
        throw new Error(
            `Dejavu native addon missing at ${file}. ` +
                "Install the matching @doki-land/dejavu-<os>-<cpu> package, " +
                "or run `node scripts/build/napi.mjs` in the dejavu-language repo, " +
                "or point DEJAVU_NATIVE_NODE at a built addon.",
        );
    }

    const require = createRequire(import.meta.url);
    cached = require(file) as NativeEngine;
    return cached;
}

/** Absolute path to this facade package root (dist/..). */
export function packageRoot(): string {
    return resolve(dirname(fileURLToPath(import.meta.url)), "..");
}

// ---------------------------------------------------------------------------
// Wire translations
// ---------------------------------------------------------------------------

function translateDiagnostic(native: NativeDiagnostic): Diagnostic {
    return {
        code: native.code,
        severity: native.severity as Severity,
        stage: native.stage,
        message: native.message,
        path: native.path ?? undefined,
        templateId: native.templateId ?? undefined,
        span: native.span
            ? ({ start: native.span.start, end: native.span.end } satisfies Span)
            : undefined,
        argsJson: native.argsJson,
        related: native.related.map((r) => ({
            start: r.start,
            end: r.end,
            label: r.label,
            path: r.path ?? undefined,
        })),
    };
}

function translateModule(native: NativeGeneratedModule): PrecompileModule {
    return {
        path: native.path,
        code: native.code,
        dependencies: [...native.dependencies],
        origins: native.origins.map((origin) => ({
            line: origin.line,
            start: origin.start,
            end: origin.end,
        })),
    };
}

/**
 * Observe non-error diagnostics, then raise the first error as a structured
 * `DejavuError` carrying the entire translated diagnostic list.
 */
function throwRenderDiagnostics(nativeDiagnostics: NativeDiagnostic[], options: RenderOptions): void {
    const diagnostics = nativeDiagnostics.map(translateDiagnostic);
    for (const diagnostic of diagnostics) {
        if (diagnostic.severity !== "error") options.onDiagnostic?.(diagnostic);
    }
    const error = DejavuError.fromDiagnostics(diagnostics);
    if (error) throw error;
}

// ---------------------------------------------------------------------------
// Low-level bridge functions (used by index.ts)
// ---------------------------------------------------------------------------

/** Parse template source into IR JSON string + diagnostics. */
export function parseIr(
    source: string,
    options: { file?: string; language?: string; delimiters?: string } = {},
): { irJson?: string; diagnostics: Diagnostic[] } {
    const native = loadNative();
    const response = native.parseIr({
        source,
        ...(options.file ? { file: options.file } : {}),
        ...(options.language ? { language: options.language } : {}),
        ...(options.delimiters ? { delimiters: options.delimiters } : {}),
    });
    return {
        irJson: response.ir,
        diagnostics: response.diagnostics.map(translateDiagnostic),
    };
}

/** Normalize IR JSON into canonical IR JSON. */
export function normalizeIr(irJson: string): { irJson?: string; diagnostics: Diagnostic[] } {
    const native = loadNative();
    const response = native.normalizeIr({ ir: irJson });
    return {
        irJson: response.ir,
        diagnostics: response.diagnostics.map(translateDiagnostic),
    };
}

/** Render an IR JSON string. */
export function renderIr(
    irJson: string,
    ctx: TemplateContext,
    options: RenderOptions = {},
): string {
    const native = loadNative();
    const response = native.renderIr({
        ir: irJson,
        context: JSON.stringify(ctx),
        options: JSON.stringify(toWireRenderOptions(options)),
    });
    throwRenderDiagnostics(response.diagnostics, options);
    return response.output ?? "";
}

/** Parse-and-render template source. */
export function renderSource(
    source: string,
    ctx: TemplateContext,
    options: RenderOptions & { delimiters?: string; catalog?: string } = {},
): string {
    const native = loadNative();
    const response = native.renderSource({
        source,
        context: JSON.stringify(ctx),
        options: JSON.stringify(toWireRenderOptions(options)),
        ...(options.delimiters ? { delimiters: options.delimiters } : {}),
        ...(options.catalog ? { catalog: options.catalog } : {}),
    });
    throwRenderDiagnostics(response.diagnostics, options);
    return response.output ?? "";
}

/** Resolve a path-v1 template reference without touching the filesystem. */
export function resolveTemplateRef(
    request: {
        reference: string;
        from?: string;
        extensions?: string[];
        roots?: Array<{
            name: string;
            scheme?: string;
            priority?: number;
            files: string[];
        }>;
    },
): { id?: string; diagnostics: Diagnostic[] } {
    const native = loadNative();
    const response = native.resolveTemplateRef(request);
    return {
        id: response.id,
        diagnostics: response.diagnostics.map(translateDiagnostic),
    };
}

/** Single-template AOT precompile. */
export function precompileIrSingle(
    irJson: string,
    options: PrecompileOptions = {},
): PrecompileResult {
    const native = loadNative();
    const response = native.precompileIrSingle({
        ir: irJson,
        options: JSON.stringify({
            strictUndefined: options.strictUndefined ?? false,
            entryId: options.entryId,
        }),
    });
    return {
        modules: response.module ? [translateModule(response.module)] : [],
        entryId: response.entryId,
        diagnostics: response.diagnostics.map(translateDiagnostic),
    };
}

/** Catalog AOT precompile (multi-template). */
export function precompileCatalog(
    entryIrJson: string,
    catalogJson: string,
    options: PrecompileOptions = {},
): PrecompileCatalogResult {
    const native = loadNative();
    const response = native.precompileCatalog({
        entryIr: entryIrJson,
        catalog: catalogJson,
        options: JSON.stringify({
            strictUndefined: options.strictUndefined ?? false,
            entryId: options.entryId,
        }),
    });
    return {
        modules: response.modules.map(translateModule),
        manifest: response.manifest ?? "",
        entryId: response.entryId,
        diagnostics: response.diagnostics.map(translateDiagnostic),
    };
}

/** Binding contract reported by the loaded addon. */
export function bindingContractVersion(): string {
    return loadNative().bindingContractVersion();
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function toWireRenderOptions(opts: RenderOptions): { strict: boolean; name?: string } {
    return {
        strict: opts.strictUndefined === true,
        ...(opts.name ? { name: opts.name } : {}),
    };
}
