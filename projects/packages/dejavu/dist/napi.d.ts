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
import type { Diagnostic, PrecompileCatalogResult, PrecompileOptions, PrecompileResult, RenderOptions, TemplateContext } from "./types.js";
export type { Diagnostic, PrecompileResult, PrecompileCatalogResult };
/** Low-level native exports; MUST match `dejavu-napi/src/lib.rs`. */
type NativeEngine = {
    bindingContractVersion(): string;
    parseIr(request: {
        source: string;
        file?: string;
        language?: string;
        delimiters?: string;
    }): {
        ir?: string;
        diagnostics: NativeDiagnostic[];
    };
    normalizeIr(request: {
        ir: string;
    }): {
        ir?: string;
        diagnostics: NativeDiagnostic[];
    };
    renderIr(request: {
        ir: string;
        context: string;
        options?: string;
        catalog?: string;
    }): {
        output?: string;
        diagnostics: NativeDiagnostic[];
    };
    renderSource(request: {
        source: string;
        context: string;
        options?: string;
        catalog?: string;
        delimiters?: string;
    }): {
        output?: string;
        diagnostics: NativeDiagnostic[];
    };
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
    }): {
        id?: string;
        diagnostics: NativeDiagnostic[];
    };
    precompileIrSingle(request: {
        ir: string;
        options?: string;
    }): {
        module?: NativeGeneratedModule;
        diagnostics: NativeDiagnostic[];
        entry_id: string;
    };
    precompileCatalog(request: {
        entry_ir: string;
        catalog: string;
        options?: string;
    }): {
        modules: NativeGeneratedModule[];
        manifest?: string;
        diagnostics: NativeDiagnostic[];
        entry_id: string;
    };
};
type NativeSpan = {
    start: number;
    end: number;
};
type NativeRelatedSpan = {
    start: number;
    end: number;
    label: string;
    path?: string | null;
};
type NativeDiagnostic = {
    code: string;
    severity: string;
    stage: string;
    message: string;
    path?: string | null;
    template_id?: string | null;
    span?: NativeSpan | null;
    args_json: string;
    related: NativeRelatedSpan[];
};
type NativeGeneratedModule = {
    path: string;
    code: string;
    dependencies: string[];
};
/** Binding contract version this facade is written against. */
export declare const BINDING_CONTRACT = "dejavu.napi/1";
/** Resolve the addon path without loading it (useful for diagnostics). */
export declare function resolveNativePath(): string;
/** Load (and cache) the native addon. Throws if the addon cannot be found. */
export declare function loadNative(): NativeEngine;
/** Absolute path to this facade package root (dist/..). */
export declare function packageRoot(): string;
/** Parse template source into IR JSON string + diagnostics. */
export declare function parseIr(source: string, options?: {
    file?: string;
    language?: string;
    delimiters?: string;
}): {
    irJson?: string;
    diagnostics: Diagnostic[];
};
/** Normalize IR JSON into canonical IR JSON. */
export declare function normalizeIr(irJson: string): {
    irJson?: string;
    diagnostics: Diagnostic[];
};
/** Render an IR JSON string. */
export declare function renderIr(irJson: string, ctx: TemplateContext, options?: RenderOptions): string;
/** Parse-and-render template source. */
export declare function renderSource(source: string, ctx: TemplateContext, options?: RenderOptions & {
    delimiters?: string;
}): string;
/** Resolve a path-v1 template reference without touching the filesystem. */
export declare function resolveTemplateRef(request: {
    reference: string;
    from?: string;
    extensions?: string[];
    roots?: Array<{
        name: string;
        scheme?: string;
        priority?: number;
        files: string[];
    }>;
}): {
    id?: string;
    diagnostics: Diagnostic[];
};
/** Single-template AOT precompile. */
export declare function precompileIrSingle(irJson: string, options?: PrecompileOptions): PrecompileResult;
/** Catalog AOT precompile (multi-template). */
export declare function precompileCatalog(entryIrJson: string, catalogJson: string, options?: PrecompileOptions): PrecompileCatalogResult;
/** Binding contract reported by the loaded addon. */
export declare function bindingContractVersion(): string;
