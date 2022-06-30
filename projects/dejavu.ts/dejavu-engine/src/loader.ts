import { parseToIr } from "@dejavu/language";
import type {
    CanonicalId,
    IrDocument,
    Language,
    TemplateDiagnostic,
    TemplateLoader,
    TemplateResolveOk,
} from "@dejavu/types";
import {
    type CatalogEntry,
    type CatalogRoot,
    canonicalId,
    posixNormalize,
    resolveTemplateRef,
} from "./resolve";

export type { CatalogRoot, CatalogEntry } from "./resolve";
export {
    resolveTemplateRef,
    posixNormalize,
    parseCanonicalId,
    canonicalId,
} from "./resolve";

export class TemplateLoaderError extends Error {
    readonly diagnostic: TemplateDiagnostic;

    constructor(diagnostic: TemplateDiagnostic) {
        super(diagnostic.message);
        this.name = "TemplateLoaderError";
        this.diagnostic = diagnostic;
    }
}

export type TemplateRootConfig = {
    name: string;
    priority: number;
    scheme?: string;
    files?: Map<string, string | IrDocument>;
    basePath?: string;
};

export type CatalogTemplateLoaderOptions = {
    roots: TemplateRootConfig[];
    extensions?: string[];
    language?: Language;
    defaultRoot?: string;
};

function toRootState(r: TemplateRootConfig): CatalogRoot {
    const files = new Map<string, CatalogEntry>();
    if (r.files) {
        for (const [k, v] of r.files) files.set(k, v as CatalogEntry);
    }
    return {
        name: r.name,
        priority: r.priority,
        scheme: r.scheme ?? r.name,
        files,
        basePath: r.basePath,
    };
}

/**
 * path-v1 catalog loader: pure resolve + parse cache.
 * `PathTemplateLoader` is an alias for host compatibility.
 */
export class CatalogTemplateLoader implements TemplateLoader {
    private readonly roots: CatalogRoot[];
    private readonly byName: Map<string, CatalogRoot>;
    private readonly byScheme: Map<string, CatalogRoot>;
    private readonly extensions: string[];
    private readonly language?: Language;
    readonly defaultRoot: string;
    private readonly cache = new Map<string, IrDocument>();

    constructor(options: CatalogTemplateLoaderOptions) {
        if (!options.roots.length) {
            throw new Error("CatalogTemplateLoader requires at least one root");
        }
        this.roots = options.roots.map(toRootState);
        this.byName = new Map(this.roots.map((r) => [r.name, r]));
        this.byScheme = new Map(this.roots.map((r) => [r.scheme, r]));
        this.extensions = options.extensions ?? ["", ".html", ".doki", ".dejavu"];
        this.language = options.language;
        const ranked = [...this.roots].sort((a, b) => b.priority - a.priority);
        this.defaultRoot = options.defaultRoot ?? ranked[0]!.name;
        if (!this.byName.has(this.defaultRoot)) {
            throw new Error(`defaultRoot not found: ${this.defaultRoot}`);
        }
    }

    get catalogRoots(): readonly CatalogRoot[] {
        return this.roots;
    }

    register(path: string, source: string | IrDocument, rootName: string = this.defaultRoot): void {
        this.set(path, source, rootName);
    }

    set(
        relativePath: string,
        source: string | IrDocument,
        rootName: string = this.defaultRoot,
    ): void {
        const root = this.byName.get(rootName);
        if (!root) throw new Error(`unknown template root: ${rootName}`);
        const key = posixNormalize(relativePath);
        root.files.set(key, source as CatalogEntry);
        this.cache.delete(canonicalId(root.name, key));
    }

    has(ref: string, options?: { from?: CanonicalId }): boolean {
        try {
            this.resolve(ref, options);
            return true;
        } catch {
            return false;
        }
    }

    list(): CanonicalId[] {
        const out: CanonicalId[] = [];
        for (const root of this.roots) {
            for (const key of root.files.keys()) {
                out.push(canonicalId(root.name, key));
            }
        }
        return out.sort();
    }

    resolve(ref: string, options?: { from?: CanonicalId }): TemplateResolveOk {
        const result = resolveTemplateRef(
            this.roots,
            this.byName,
            this.byScheme,
            this.extensions,
            ref,
            options?.from,
        );
        if (!result.ok) {
            throw new TemplateLoaderError(result.diagnostic);
        }
        const { root, path } = result.hit;
        const id = canonicalId(root.name, path);
        let document = this.cache.get(id);
        if (!document) {
            const entry = root.files.get(path)!;
            document =
                typeof entry === "string"
                    ? parseToIr(entry, { file: id, language: this.language })
                    : (entry as unknown as IrDocument);
            this.cache.set(id, document);
        }
        const sourcePath = root.basePath
            ? `${root.basePath.replace(/\\/g, "/").replace(/\/$/, "")}/${path}`
            : undefined;
        return { id, root: root.name, path, sourcePath, document };
    }

    load(ref: string, options?: { from?: CanonicalId }): IrDocument {
        return this.resolve(ref, options).document;
    }
}

/** Alias — same implementation as CatalogTemplateLoader. */
export class PathTemplateLoader extends CatalogTemplateLoader {}

export type PathTemplateLoaderOptions = CatalogTemplateLoaderOptions;

/**
 * Single-root degeneration of path-v1 (`map:`).
 * Bare names resolve via the `map` root; canonical ids are `map:{path}`.
 */
export class MapTemplateLoader extends CatalogTemplateLoader {
    constructor(templates: Map<string, string | IrDocument>, options?: { language?: Language }) {
        super({
            language: options?.language,
            defaultRoot: "map",
            extensions: [""],
            roots: [
                {
                    name: "map",
                    priority: 1,
                    scheme: "map",
                    files: templates,
                },
            ],
        });
    }

    static fromRecord(
        record: Record<string, string | IrDocument>,
        options?: { language?: Language },
    ): MapTemplateLoader {
        return new MapTemplateLoader(new Map(Object.entries(record)), options);
    }
}
