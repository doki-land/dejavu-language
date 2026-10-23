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
// ---------------------------------------------------------------------------
// Addon loading
// ---------------------------------------------------------------------------
/** Binding contract version this facade is written against. */
export const BINDING_CONTRACT = "dejavu.napi/1";
const PLATFORM_PACKAGES = {
    "linux-x64": "@doki-land/dejavu-linux-x64",
    "linux-arm64": "@doki-land/dejavu-linux-arm64",
    "darwin-x64": "@doki-land/dejavu-darwin-x64",
    "darwin-arm64": "@doki-land/dejavu-darwin-arm64",
    "win32-x64": "@doki-land/dejavu-win32-x64",
    "win32-arm64": "@doki-land/dejavu-win32-arm64",
};
let cached;
/** Resolve the addon path without loading it (useful for diagnostics). */
export function resolveNativePath() {
    const override = process.env.DEJAVU_NATIVE_NODE;
    if (override) {
        return isAbsolute(override) ? override : resolve(override);
    }
    const key = `${process.platform}-${process.arch}`;
    const pkg = PLATFORM_PACKAGES[key];
    if (!pkg) {
        throw new Error(`Dejavu native addon does not support platform ${key}. ` +
            "Set DEJAVU_NATIVE_NODE to a built addon or add a platform package.");
    }
    const require = createRequire(import.meta.url);
    let pkgJson;
    try {
        pkgJson = require.resolve(`${pkg}/package.json`);
    }
    catch {
        throw new Error(`Dejavu platform package ${pkg} is not installed (platform ${key}). ` +
            "Reinstall @doki-land/dejavu with optional dependencies enabled.");
    }
    const pkgDir = dirname(pkgJson);
    const meta = require(pkgJson);
    return join(pkgDir, meta.main ?? "index.node");
}
/** Load (and cache) the native addon. Throws if the addon cannot be found. */
export function loadNative() {
    if (cached)
        return cached;
    const file = resolveNativePath();
    if (!existsSync(file)) {
        throw new Error(`Dejavu native addon missing at ${file}. ` +
            "Install the matching @doki-land/dejavu-<os>-<cpu> package, " +
            "or run `node scripts/build/napi.mjs` in the dejavu-language repo, " +
            "or point DEJAVU_NATIVE_NODE at a built addon.");
    }
    const require = createRequire(import.meta.url);
    cached = require(file);
    return cached;
}
/** Absolute path to this facade package root (dist/..). */
export function packageRoot() {
    return resolve(dirname(fileURLToPath(import.meta.url)), "..");
}
// ---------------------------------------------------------------------------
// Wire translations
// ---------------------------------------------------------------------------
function translateDiagnostic(native) {
    return {
        code: native.code,
        severity: native.severity,
        stage: native.stage,
        message: native.message,
        path: native.path ?? undefined,
        templateId: native.template_id ?? undefined,
        span: native.span
            ? { start: native.span.start, end: native.span.end }
            : undefined,
        argsJson: native.args_json,
        related: native.related.map((r) => ({
            start: r.start,
            end: r.end,
            label: r.label,
            path: r.path ?? undefined,
        })),
    };
}
function translateModule(native) {
    return {
        path: native.path,
        code: native.code,
        dependencies: [...native.dependencies],
    };
}
// ---------------------------------------------------------------------------
// Low-level bridge functions (used by index.ts)
// ---------------------------------------------------------------------------
/** Parse template source into IR JSON string + diagnostics. */
export function parseIr(source, options = {}) {
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
export function normalizeIr(irJson) {
    const native = loadNative();
    const response = native.normalizeIr({ ir: irJson });
    return {
        irJson: response.ir,
        diagnostics: response.diagnostics.map(translateDiagnostic),
    };
}
/** Render an IR JSON string. */
export function renderIr(irJson, ctx, options = {}) {
    const native = loadNative();
    const response = native.renderIr({
        ir: irJson,
        context: JSON.stringify(ctx),
        options: JSON.stringify(toWireRenderOptions(options)),
    });
    for (const d of response.diagnostics) {
        if (d.severity === "error") {
            throw new Error(`dejavu: renderIr error — ${d.code}: ${d.message}`);
        }
        const translated = translateDiagnostic(d);
        options.onDiagnostic?.(translated);
    }
    return response.output ?? "";
}
/** Parse-and-render template source. */
export function renderSource(source, ctx, options = {}) {
    const native = loadNative();
    const response = native.renderSource({
        source,
        context: JSON.stringify(ctx),
        options: JSON.stringify(toWireRenderOptions(options)),
        ...(options.delimiters ? { delimiters: options.delimiters } : {}),
    });
    for (const d of response.diagnostics) {
        if (d.severity === "error") {
            throw new Error(`dejavu: renderSource error — ${d.code}: ${d.message}`);
        }
        const translated = translateDiagnostic(d);
        options.onDiagnostic?.(translated);
    }
    return response.output ?? "";
}
/** Resolve a path-v1 template reference without touching the filesystem. */
export function resolveTemplateRef(request) {
    const native = loadNative();
    const response = native.resolveTemplateRef(request);
    return {
        id: response.id,
        diagnostics: response.diagnostics.map(translateDiagnostic),
    };
}
/** Single-template AOT precompile. */
export function precompileIrSingle(irJson, options = {}) {
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
        entryId: response.entry_id,
        diagnostics: response.diagnostics.map(translateDiagnostic),
    };
}
/** Catalog AOT precompile (multi-template). */
export function precompileCatalog(entryIrJson, catalogJson, options = {}) {
    const native = loadNative();
    const response = native.precompileCatalog({
        entry_ir: entryIrJson,
        catalog: catalogJson,
        options: JSON.stringify({
            strictUndefined: options.strictUndefined ?? false,
            entryId: options.entryId,
        }),
    });
    return {
        modules: response.modules.map(translateModule),
        manifest: response.manifest ?? "",
        entryId: response.entry_id,
        diagnostics: response.diagnostics.map(translateDiagnostic),
    };
}
/** Binding contract reported by the loaded addon. */
export function bindingContractVersion() {
    return loadNative().bindingContractVersion();
}
// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
function toWireRenderOptions(opts) {
    return {
        strict: opts.strictUndefined === true,
        ...(opts.name ? { name: opts.name } : {}),
    };
}
