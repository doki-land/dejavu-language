/**
 * Public types for the `@doki-land/dejavu` facade.
 *
 * IR and diagnostics are defined by the Rust engine (`dejavu-ir` / `dejavu-diagnostic`);
 * these types only describe the N-API wire shape.
 */

/** UTF-8 byte span, `end` exclusive. */
export type DejavuSpan = {
    start: number;
    end: number;
};

/** Related source location carried by a diagnostic. */
export type DejavuRelatedSpan = {
    start: number;
    end: number;
    label: string;
    path?: string | null;
};

/** Stable-severity alias mirrored from `dejavu-diagnostic::Severity`. */
export type DejavuSeverity = "error" | "warning" | "info" | "debug";

/** Structured engine diagnostic. */
export type DejavuDiagnostic = {
    /** Stable machine identity, e.g. `dejavu.template.template-not-found`. */
    code: string;
    severity: DejavuSeverity | string;
    /** `parse` / `load` / `resolve` / `render` / `compile` / `config` / `host`. */
    stage: string;
    /** Fallback text; catalogs translate from `code` + `argsJson`. */
    message: string;
    path?: string | null;
    templateId?: string | null;
    span?: DejavuSpan | null;
    /** Structured args as a JSON object string. */
    argsJson: string;
    related: DejavuRelatedSpan[];
};

/** Parsed template IR. Shape is owned by the Rust engine. */
export type IrDocument = Record<string, unknown>;

/** Template render context. */
export type TemplateContext = Record<string, unknown>;

/** Render options accepted by `render` / `renderSource`. */
export type DejavuRenderOptions = {
    /** Fail on undefined variables instead of rendering empty text. */
    strict?: boolean;
    /** Canonical id used for diagnostics and relative references. */
    name?: string;
};

/** One catalog root. `files` maps canonical paths to IR documents or IR JSON. */
export type DejavuCatalogRoot = {
    name: string;
    scheme?: string;
    priority?: number;
    basePath?: string;
    files: Record<string, IrDocument | string>;
};

/** Catalog input accepted by the facade and serialized for N-API. */
export type DejavuCatalog = {
    roots: DejavuCatalogRoot[];
    extensions?: string[];
};

/** Reference resolution request. */
export type ResolveTemplateRefRequest = {
    reference: string;
    from?: string;
    extensions?: string[];
    roots?: Array<{
        name: string;
        scheme?: string;
        priority?: number;
        files: string[];
    }>;
};

/** Reference resolution result: `id` is `root:path` on success. */
export type ResolveTemplateRefResult = {
    id?: string;
    diagnostics: DejavuDiagnostic[];
};
