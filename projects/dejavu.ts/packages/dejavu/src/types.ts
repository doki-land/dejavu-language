/**
 * Public types for the `@doki-land/dejavu` facade.
 *
 * These types describe the author-facing contract documented in
 * `dejavu-typescript-ux.md`. N-API wire types live in `napi.ts`
 * and are translated here into stable camelCase shapes.
 */

// ---------------------------------------------------------------------------
// Diagnostics
// ---------------------------------------------------------------------------

/** UTF-8 byte span, `end` exclusive. */
export type Span = {
    start: number;
    end: number;
};

/** Related source location carried by a diagnostic. */
export type RelatedSpan = {
    start: number;
    end: number;
    label: string;
    path?: string;
};

/** Stable severity alias mirrored from `dejavu-diagnostic::Severity`. */
export type Severity = "error" | "warning" | "info" | "debug";

/** Structured engine diagnostic. */
export type Diagnostic = {
    /** Stable machine identity, e.g. `dejavu.template.template-not-found`. */
    code: string;
    severity: Severity;
    /** `parse` / `load` / `resolve` / `render` / `compile` / `config` / `host`. */
    stage: string;
    /** Fallback text; catalogs translate from `code` + `args`. */
    message: string;
    path?: string;
    templateId?: string;
    span?: Span;
    /** Structured args as a JSON object string. */
    argsJson: string;
    related: RelatedSpan[];
};

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/** Options accepted by `render` / `compile` / `renderSource`. */
export type RenderOptions = {
    /** Fail on undefined variables instead of rendering empty text. */
    strictUndefined?: boolean;
    /** Canonical id used for diagnostics and relative references. */
    name?: string;
    /**
     * Registry capabilities granted to this render (`pure` / `safe-output`).
     * Omit to grant everything; an entry outside the list is rejected.
     */
    capabilities?: string[];
    /** Optional onDiagnostic callback for observing warnings. */
    onDiagnostic?: (d: Diagnostic) => void;
    /** Optional onDependency callback for observing resolved include/extends. */
    onDependency?: (id: string) => void;
};

/** Options accepted by `parse`. */
export type ParseOptions = {
    /** Canonical source path for diagnostics. */
    file?: string;
    /** Optional IR-language identifier. Currently only T1 is supported. */
    language?: string;
    /** Custom delimiter spec (JSON string). */
    delimiters?: string;
};

/** Options accepted by `precompile`. */
export type PrecompileOptions = {
    /** Fail on undefined variables in generated code. */
    strictUndefined?: boolean;
    /** Entry template id used for dependency tracking. */
    entryId?: string;
    /** Registry capabilities the compiled modules may use. Omit to grant all. */
    capabilities?: string[];
};

/** Options accepted by `compile` (JIT preparation). */
export type CompileOptions = {
    /** Parse-time canonical source path for diagnostics. */
    file?: string;
    /** Custom delimiter spec (JSON string). */
    delimiters?: string;
    /** Default render option: fail on undefined variables. */
    strictUndefined?: boolean;
    /** Default render option: canonical id for diagnostics. */
    name?: string;
};

// ---------------------------------------------------------------------------
// Template context
// ---------------------------------------------------------------------------

/**
 * Template render context. Values MUST be JSON-safe: no functions, Symbols,
 * BigInt, undefined, circular refs, or non-finite Numbers.
 */
export type TemplateContext = Record<string, unknown>;

// ---------------------------------------------------------------------------
// Catalog
// ---------------------------------------------------------------------------

/** One catalog root. `files` maps relative canonical paths to template sources. */
export type CatalogRoot = {
    name: string;
    scheme?: string;
    priority?: number;
    basePath?: string;
    files: Record<string, string>;
};

/** Catalog input accepted by the facade and serialized for N-API. */
export type Catalog = {
    roots: CatalogRoot[];
    extensions?: string[];
};

/** Reference resolution request (path-v1, no filesystem scan). */
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
    diagnostics: Diagnostic[];
};

// ---------------------------------------------------------------------------
// JIT Template (compile result)
// ---------------------------------------------------------------------------

/**
 * A JIT-prepared template returned by `compile()`.
 *
 * Templates are **process-local** objects; they are NOT serializable,
 * distributable, or safe to pass across Worker boundaries. Each `.render(ctx)`
 * is independent — no context/block state leaks between calls.
 */
export class Template {
    private readonly _irJson: string;
    private readonly _defaultOptions: CompileOptions;
    private readonly _render: (
        irJson: string,
        ctx: TemplateContext,
        options: RenderOptions,
    ) => string;

    constructor(
        irJson: string,
        defaultOptions: CompileOptions,
        render: (irJson: string, ctx: TemplateContext, options: RenderOptions) => string,
    ) {
        this._irJson = irJson;
        this._defaultOptions = defaultOptions;
        this._render = render;
    }

    /** The compiled IR as a JSON string. Intended for debugging. */
    get irJson(): string {
        return this._irJson;
    }

    /** Re-render with the stored JIT preparation. Source is NOT re-parsed. */
    render(ctx: TemplateContext = {}, options: RenderOptions = {}): string {
        return this._render(this._irJson, ctx, { ...this._defaultOptions, ...options });
    }
}

// ---------------------------------------------------------------------------
// AOT Precompile results
// ---------------------------------------------------------------------------

/** Where one generated JS line starts inside the template source. */
export type GeneratedOrigin = {
    /** 0-based generated module line. */
    line: number;
    /** Template byte offset where the construct starts. */
    start: number;
    /** Template byte offset where the construct ends (exclusive). */
    end: number;
};

/** One generated ESM module from AOT codegen. */
export type PrecompileModule = {
    /** Relative path inside the output root, e.g. `"index.js"`. */
    path: string;
    /** ESM JavaScript source code. */
    code: string;
    /** Canonical ids of templates this module imports (via `include`/`extends`). */
    dependencies: string[];
    /** Generated line → template span points, used to build source maps. */
    origins: GeneratedOrigin[];
};

/** Result of a single-template AOT precompile. */
export type PrecompileResult = {
    /** Generated ESM module(s). Single-template precompile returns exactly one module. */
    modules: PrecompileModule[];
    /** Canonical entry template id (e.g. `"__entry__"` or an explicit id). */
    entryId: string;
    /** All diagnostics collected during compilation. */
    diagnostics: Diagnostic[];
    /** Placeholder: .d.ts declarations for the generated render function (NOT YET PRODUCED). */
    declarations?: string;
    /** Placeholder: source map JSON string mapping generated JS back to template source (NOT YET PRODUCED). */
    sourceMap?: string;
};

/** Result of a catalog AOT precompile (multi-template). */
export type PrecompileCatalogResult = {
    /** All generated modules (entry first, then internal modules in deterministic order). */
    modules: PrecompileModule[];
    /** Canonical entry template id. */
    entryId: string;
    /** JSON manifest describing entry id, entry module path, and canonical-id → file-path map. */
    manifest: string;
    /** All diagnostics collected during compilation. */
    diagnostics: Diagnostic[];
    /** Placeholder: .d.ts declarations (NOT YET PRODUCED). */
    declarations?: string;
    /** Placeholder: source map JSON (NOT YET PRODUCED). */
    sourceMap?: string;
};
