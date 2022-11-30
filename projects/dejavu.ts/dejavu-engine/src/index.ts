import {parseToIr} from "@dejavu/language";
import {
    normalizeIrJson,
    type IrDocument,
    type IrValue,
    type Language,
    type RenderOptions,
    type TemplateLoader,
} from "@dejavu/types";
import {MapTemplateLoader} from "./loader";
import {renderIr} from "./render";

export {renderIr} from "./render";
export {
    CatalogTemplateLoader,
    MapTemplateLoader,
    PathTemplateLoader,
    TemplateLoaderError,
} from "./loader";
export type {
    TemplateRootConfig,
    PathTemplateLoaderOptions,
    CatalogTemplateLoaderOptions,
} from "./loader";
export {
    resolveTemplateRef,
    posixNormalize,
    parseCanonicalId,
    canonicalId,
} from "./resolve";
export {parseToIr} from "@dejavu/language";
export {
    normalizeIrJson,
    normalizeValue,
    DEFAULT_LANGUAGE,
    markSafe,
    isSafeHtml,
    htmlEscape,
    LOADER_RESOLUTION_PATH_V1,
    type IrDocument,
    type IrNode,
    type IrValue,
    type Language,
    type TemplateConfig,
    type RenderOptions,
    type TemplateLoader,
    type TemplateLoadResult,
    type TemplateResolveOk,
    type TemplateDiagnostic,
    type TemplateDiagnosticCode,
    type CanonicalId,
    type ParseOptions,
    type SafeHtmlValue,
} from "@dejavu/types";

export class DejavuEngine {
    private language: Language | undefined;
    private loader: TemplateLoader;

    constructor(options?: { language?: Language; loader?: TemplateLoader }) {
        this.language = options?.language;
        this.loader =
            options?.loader ??
            new MapTemplateLoader(new Map(), {language: this.language});
    }

    setLanguage(language: Language): this {
        this.language = language;
        return this;
    }

    setLoader(loader: TemplateLoader): this {
        this.loader = loader;
        return this;
    }

    getLoader(): TemplateLoader {
        return this.loader;
    }

    /**
     * Register a template via `loader.register` when available.
     * path-v1: `name` is a relative path under the default root, or `scheme:rel`.
     */
    registerTemplate(name: string, source: string, language?: Language): this {
        const lang = language ?? this.language;
        const schemeMatch = /^([A-Za-z][A-Za-z0-9_+-]*):(.*)$/.exec(name);

        if (this.loader.register) {
            if (schemeMatch) {
                this.loader.register(schemeMatch[2]!, source, schemeMatch[1]!);
            } else {
                this.loader.register(name, source);
            }
            return this;
        }

        const doc = parseToIr(source, {file: name, language: lang});
        this.loader = new MapTemplateLoader(new Map([[name, doc]]), {language: lang});
        return this;
    }

    parse(source: string, fileOrOpts?: string | { file?: string; language?: Language }): IrDocument {
        if (typeof fileOrOpts === "string") {
            return parseToIr(source, {file: fileOrOpts, language: this.language});
        }
        return parseToIr(source, {
            file: fileOrOpts?.file,
            language: fileOrOpts?.language ?? this.language,
        });
    }

    render(
        ir: IrDocument | string,
        ctx: Record<string, IrValue> = {},
        options: RenderOptions = {},
    ): string {
        const doc = typeof ir === "string" ? (JSON.parse(ir) as IrDocument) : ir;
        return renderIr(doc, ctx, {
            loader: options.loader ?? this.loader,
            name: options.name,
            strictUndefined: options.strictUndefined,
            onDependency: options.onDependency,
        });
    }

    renderSource(
        source: string,
        ctx: Record<string, IrValue> = {},
        options: RenderOptions & { language?: Language } = {},
    ): string {
        const name = options.name ?? "<main>";
        const doc = this.parse(source, {file: name, language: options.language});
        return this.render(doc, ctx, {...options, name});
    }

    renderTemplate(
        name: string,
        ctx: Record<string, IrValue> = {},
        options: Pick<RenderOptions, "onDependency" | "strictUndefined"> = {},
    ): string {
        const loaded = this.loader.resolve(name);
        options.onDependency?.(loaded.id);
        return renderIr(loaded.document, ctx, {
            loader: this.loader,
            name: loaded.id,
            strictUndefined: options.strictUndefined,
            onDependency: options.onDependency,
        });
    }

    check(source: string, language?: Language): { valid: boolean; errors: string[] } {
        try {
            parseToIr(source, {language: language ?? this.language});
            return {valid: true, errors: []};
        } catch (e) {
            return {valid: false, errors: [String(e)]};
        }
    }

    /** Compatibility alias used by Doki's check path. */
    checkTemplate(source: string, language?: Language): { valid: boolean; errors: string[] } {
        return this.check(source, language);
    }
}

export const engine = new DejavuEngine();

export function deepEqualNormalized(a: unknown, b: unknown): boolean {
    return JSON.stringify(a) === JSON.stringify(b);
}

export function compareIrJson(got: string, expected: string): boolean {
    return deepEqualNormalized(normalizeIrJson(got), normalizeIrJson(expected));
}
