/**
 * Engine facade over `dejavu-napi`.
 *
 * Every function here is a thin, typed composition of one native capability.
 * Command semantics (argv, config, stdio, exit codes) belong to `src/cli/`.
 */

export { DejavuError } from "./errors.js";
export {
    BINDING_CONTRACT,
    loadNative,
    packageRoot,
    resolveNativePath,
    type NativeEngine,
} from "./native.js";
export { loadCatalogFromDirs, catalogToJson, type CatalogDir } from "./catalog.js";
export type {
    DejavuCatalog,
    DejavuCatalogRoot,
    DejavuDiagnostic,
    DejavuRelatedSpan,
    DejavuRenderOptions,
    DejavuSeverity,
    DejavuSpan,
    IrDocument,
    ResolveTemplateRefRequest,
    ResolveTemplateRefResult,
    TemplateContext,
} from "./types.js";

import { catalogToJson } from "./catalog.js";
import { DejavuError } from "./errors.js";
import { loadNative, type NativeEngine } from "./native.js";
import type {
    DejavuCatalog,
    DejavuDiagnostic,
    DejavuRenderOptions,
    IrDocument,
    ResolveTemplateRefRequest,
    ResolveTemplateRefResult,
    TemplateContext,
} from "./types.js";

/** Parse template source into IR, throwing on error diagnostics. */
export function parse(
    source: string,
    options: { file?: string; language?: string; delimiters?: unknown; engine?: NativeEngine } = {},
): IrDocument {
    const engine = options.engine ?? loadNative();
    const response = engine.parseIr({
        source,
        ...(options.file === undefined ? {} : { file: options.file }),
        ...(options.language === undefined ? {} : { language: options.language }),
        ...(options.delimiters === undefined ? {} : { delimiters: JSON.stringify(options.delimiters) }),
    });
    throwIfErrors(response.diagnostics);
    if (typeof response.ir !== "string") {
        throw new Error("dejavu: parseIr returned no IR and no error diagnostic");
    }
    return JSON.parse(response.ir) as IrDocument;
}

/** Normalize an IR document (or IR JSON) into its canonical JSON form. */
export function normalizeIr(ir: IrDocument | string, engine?: NativeEngine): IrDocument {
    const native = engine ?? loadNative();
    const response = native.normalizeIr({ ir: typeof ir === "string" ? ir : JSON.stringify(ir) });
    throwIfErrors(response.diagnostics);
    if (typeof response.ir !== "string") {
        throw new Error("dejavu: normalizeIr returned no IR and no error diagnostic");
    }
    return JSON.parse(response.ir) as IrDocument;
}

/** Render an IR document or IR JSON string. */
export function render(
    ir: IrDocument | string,
    context: TemplateContext = {},
    options: DejavuRenderOptions & { catalog?: DejavuCatalog; engine?: NativeEngine } = {},
): string {
    const engine = options.engine ?? loadNative();
    const response = engine.renderIr({
        ir: typeof ir === "string" ? ir : JSON.stringify(ir),
        context: JSON.stringify(context),
        options: JSON.stringify(renderOptions(options)),
        ...(options.catalog === undefined ? {} : { catalog: catalogToJson(options.catalog) }),
    });
    throwIfErrors(response.diagnostics);
    return response.output ?? "";
}

/** Parse and render template source in one step. */
export function renderSource(
    source: string,
    context: TemplateContext = {},
    options: DejavuRenderOptions & { catalog?: DejavuCatalog; delimiters?: unknown; engine?: NativeEngine } = {},
): string {
    const engine = options.engine ?? loadNative();
    const response = engine.renderSource({
        source,
        context: JSON.stringify(context),
        options: JSON.stringify(renderOptions(options)),
        ...(options.catalog === undefined ? {} : { catalog: catalogToJson(options.catalog) }),
        ...(options.delimiters === undefined ? {} : { delimiters: JSON.stringify(options.delimiters) }),
    });
    throwIfErrors(response.diagnostics);
    return response.output ?? "";
}

/** Resolve a path-v1 template reference without touching the filesystem. */
export function resolveTemplateRef(
    request: ResolveTemplateRefRequest,
    engine?: NativeEngine,
): ResolveTemplateRefResult {
    const native = engine ?? loadNative();
    return native.resolveTemplateRef(request);
}

/** Binding contract reported by the loaded addon. */
export function bindingContractVersion(engine?: NativeEngine): string {
    return (engine ?? loadNative()).bindingContractVersion();
}

function renderOptions(options: DejavuRenderOptions): { strict: boolean; name?: string } {
    return {
        strict: options.strict === true,
        ...(options.name === undefined ? {} : { name: options.name }),
    };
}

function throwIfErrors(diagnostics: DejavuDiagnostic[]): void {
    const error = DejavuError.fromDiagnostics(diagnostics);
    if (error) throw error;
}
