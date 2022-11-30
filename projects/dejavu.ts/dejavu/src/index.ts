/**
 * Public TypeScript surface: `import { Dejavu, parse, render } from "dejavu"`.
 *
 * Internal packages (`@dejavu/language`, `@dejavu/types`, `@dejavu/engine`) are
 * implementation details — app code should depend on **`dejavu`** only.
 */

export {
    DejavuEngine,
    engine,
    parseToIr as parse,
    renderIr as render,
    MapTemplateLoader,
    PathTemplateLoader,
    CatalogTemplateLoader,
    TemplateLoaderError,
    normalizeIrJson,
    normalizeValue,
    DEFAULT_LANGUAGE,
    LOADER_RESOLUTION_PATH_V1,
    compareIrJson,
    deepEqualNormalized,
    markSafe,
    isSafeHtml,
    htmlEscape,
    type IrDocument,
    type IrNode,
    type IrValue,
    type Language,
    type RenderOptions,
    type TemplateLoader,
    type TemplateLoadResult,
    type TemplateResolveOk,
    type CanonicalId,
    type ParseOptions,
    type SafeHtmlValue,
} from "@dejavu/engine";

import {
    DejavuEngine,
    MapTemplateLoader,
    PathTemplateLoader,
    CatalogTemplateLoader,
    parseToIr,
    renderIr,
    type IrDocument,
    type IrValue,
    type Language,
    type RenderOptions,
} from "@dejavu/engine";

/** Canonical user-facing facade (mirrors `Dejavu` in other hosts). */
export const Dejavu = {
    parse(source: string, options?: { file?: string; language?: Language }): IrDocument {
        return parseToIr(source, options);
    },
    render(
        ir: IrDocument | string,
        ctx: Record<string, IrValue> = {},
        options: RenderOptions = {},
    ): string {
        const doc = typeof ir === "string" ? (JSON.parse(ir) as IrDocument) : ir;
        return renderIr(doc, ctx, options);
    },
    renderSource(
        source: string,
        ctx: Record<string, IrValue> = {},
        options: RenderOptions = {},
    ): string {
        const name = options.name ?? "<main>";
        return renderIr(parseToIr(source, {file: name}), ctx, {...options, name});
    },
    check(source: string): { valid: boolean; errors: string[] } {
        return new DejavuEngine().check(source);
    },
    /** Create an engine with an in-memory template registry (extends/include). */
    withTemplates(
        templates: Record<string, string>,
        options?: { language?: Language },
    ): DejavuEngine {
        const eng = new DejavuEngine({language: options?.language});
        for (const [name, source] of Object.entries(templates)) {
            eng.registerTemplate(name, source);
        }
        return eng;
    },
    MapTemplateLoader,
    PathTemplateLoader,
    CatalogTemplateLoader,
};

export default Dejavu;
