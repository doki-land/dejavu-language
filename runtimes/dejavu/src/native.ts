/**
 * Load the `dejavu-napi` native addon shipped by a platform package.
 *
 * Resolution order:
 * 1. `DEJAVU_NATIVE_NODE` (absolute path or resolvable module id) — local dev and tests.
 * 2. `@doki-land/dejavu-<os>-<cpu>` optional dependency installed by npm.
 *
 * There is deliberately no TypeScript fallback parser: the Rust engine is the only engine.
 */

import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, isAbsolute, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import type {
    DejavuCatalog,
    DejavuDiagnostic,
    DejavuRenderOptions,
    IrDocument,
    ResolveTemplateRefRequest,
    ResolveTemplateRefResult,
    TemplateContext,
} from "./types.js";

/** Native export surface; must match `dejavu-napi` and the binding contract. */
export type NativeEngine = {
    bindingContractVersion(): string;
    parseIr(request: { source: string; file?: string; language?: string }): {
        ir?: string;
        diagnostics: DejavuDiagnostic[];
    };
    normalizeIr(request: { ir: string }): { ir?: string; diagnostics: DejavuDiagnostic[] };
    renderIr(request: { ir: string; context: string; options?: string; catalog?: string }): {
        output?: string;
        diagnostics: DejavuDiagnostic[];
    };
    renderSource(request: {
        source: string;
        context: string;
        options?: string;
        catalog?: string;
    }): { output?: string; diagnostics: DejavuDiagnostic[] };
    resolveTemplateRef(request: ResolveTemplateRefRequest): ResolveTemplateRefResult;
};

/** Binding contract this facade is written against. */
export const BINDING_CONTRACT = "dejavu.napi/1";

const PLATFORM_PACKAGES: Record<string, string> = {
    "linux-x64": "@doki-land/dejavu-linux-x64",
    "linux-arm64": "@doki-land/dejavu-linux-arm64",
    "darwin-x64": "@doki-land/dejavu-darwin-x64",
    "darwin-arm64": "@doki-land/dejavu-darwin-arm64",
    "win32-x64": "@doki-land/dejavu-win32-x64",
    "win32-arm64": "@doki-land/dejavu-win32-arm64",
};

let cached: NativeEngine | undefined;

/** Load (and cache) the native addon. */
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

/** Absolute path to this facade package root (dist/..). */
export function packageRoot(): string {
    return resolve(dirname(fileURLToPath(import.meta.url)), "..");
}

// Re-export wire helper types used by callers building native requests.
export type { DejavuCatalog, DejavuRenderOptions, IrDocument, TemplateContext };
