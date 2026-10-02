/** Template loading + render options for inheritance / inclusion (path-v1). */

import type { IrDocument, IrValue, Language } from "./ir";

/** Capability id for the Loader & Resolution chapter. */
export const LOADER_RESOLUTION_PATH_V1 = "path-v1" as const;
export type LoaderResolutionCapability = typeof LOADER_RESOLUTION_PATH_V1;

/**
 * Stable template identity: `{rootName}:{posixRelPath}`.
 * Used for cycle detection, relative resolve `from`, and dependency recording.
 */
export type CanonicalId = string;

export type TemplateDiagnosticCode =
    | "template_not_found"
    | "relative_without_from"
    | "relative_escape_root"
    | "unknown_scheme"
    | "invalid_ref";

export interface TemplateDiagnostic {
    code: TemplateDiagnosticCode;
    message: string;
    ref: string;
    from?: CanonicalId;
    /** Root names searched (bare miss). */
    searched?: string[];
}

/**
 * Successful resolve: canonical identity + parsed IR.
 * `TemplateLoadResult` is kept as a compatibility alias.
 */
export interface TemplateResolveOk {
    id: CanonicalId;
    root: string;
    path: string;
    sourcePath?: string;
    document: IrDocument;
}

/** @deprecated Prefer TemplateResolveOk — same shape plus root/path. */
export type TemplateLoadResult = TemplateResolveOk;

/**
 * path-v1 loader: `resolve` is required.
 * `load` is a convenience for `resolve(...).document`.
 * Optional `register` lets hosts/`DejavuEngine` add templates without instanceof.
 */
export interface TemplateLoader {
    resolve(ref: string, options?: { from?: CanonicalId }): TemplateResolveOk;

    load(ref: string, options?: { from?: CanonicalId }): IrDocument;

    register?(path: string, source: string | IrDocument, root?: string): void;

    has?(ref: string, options?: { from?: CanonicalId }): boolean;

    list?(): CanonicalId[];
}

export interface RenderOptions {
    /** Required when the IR contains Extends / Include / Super. */
    loader?: TemplateLoader;
    /**
     * Canonical id of the document being rendered (cycle detection / diagnostics).
     * Defaults to `"<main>"`. Prefer a CanonicalId from `loader.resolve`.
     */
    name?: CanonicalId;
    /**
     * When true, reading an unbound identifier throws.
     * Default false: missing → `null` → empty interpolation (Contract 1.0).
     */
    strictUndefined?: boolean;
    /** Invoked for each successfully resolved template id during render (incl. entry). */
    onDependency?: (id: CanonicalId) => void;
}

export type ParseOptions = {
    file?: string;
    language?: Language;
};

export type { IrDocument, IrValue, Language };
