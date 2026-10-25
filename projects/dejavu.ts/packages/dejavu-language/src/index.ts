/**
 * Dejavu WASM language bridge — host wrapper.
 *
 * The Rust crate `dejavu-wasm` is compiled to `wasm32-unknown-unknown` and
 * shipped by `@doki-land/dejavu-unknown-wasm32`. This package owns the host
 * contract: initialization for browsers, Workers and Node, typed request and
 * response shapes, and JSON encoding at the bridge boundary.
 *
 * Every capability mirrors the desktop engine surface, so a web host never
 * needs a Node addon, the filesystem or a compiler.
 */
import * as glue from "@doki-land/dejavu-unknown-wasm32";

/** Binding contract reported by the bridge. */
export const BINDING_CONTRACT = "dejavu.wasm/2";

/** Diagnostic severity. */
export type Severity = "error" | "warning" | "info" | "debug";

/** UTF-8 byte range in the template source, `end` exclusive. */
export interface Span {
    start: number;
    end: number;
}

/** A related location carried by a diagnostic. */
export interface RelatedSpan extends Span {
    label: string;
    path?: string;
}

/** Language-neutral diagnostic wire. */
export interface Diagnostic {
    code: string;
    severity: Severity;
    stage: string;
    message: string;
    path?: string;
    templateId?: string;
    span?: Span;
    /** JSON object string with structured arguments, prefer it over `message`. */
    argsJson: string;
    related: RelatedSpan[];
}

/** Delimiters override, matching the IR `Language` shape. */
export interface Language {
    expression?: [string, string];
    statement?: [string, string];
}

/** Render options shared by `renderIr` and `renderSource`. */
export interface RenderOptions {
    strictUndefined?: boolean;
    name?: string;
    /** Registry capabilities granted to this render (`pure` / `safe-output`). */
    capabilities?: string[];
}

/**
 * One host-supplied registry entry.
 *
 * The description mirrors the Rust `CustomEntry` wire, so conflict, arity and
 * capability checks stay identical across hosts. Implementations live in
 * {@link CustomRegistry} keyed by canonical name.
 */
export interface CustomRegistryEntry {
    namespace: "filter" | "function" | "test";
    name: string;
    aliases?: string[];
    minArgs?: number;
    maxArgs?: number;
    capability?: "pure" | "safe-output";
}

/** Host-supplied implementations; never mutated during a render. */
export interface CustomRegistry {
    entries: CustomRegistryEntry[];
    filter?: Record<string, (value: unknown, args: unknown[]) => unknown>;
    function?: Record<string, (args: unknown[]) => unknown>;
    test?: Record<string, (value: unknown, args: unknown[]) => boolean>;
}

/** In-memory template catalog for inheritance and includes. */
export interface CatalogSpec {
    roots: CatalogRoot[];
    extensions?: string[];
}

/** One catalog root. */
export interface CatalogRoot {
    name: string;
    scheme?: string;
    priority?: number;
    files: Record<string, string>;
    basePath?: string;
}

/** Parse / normalize result. */
export interface IrResult {
    ir?: string;
    diagnostics: Diagnostic[];
}

/** Render result. */
export interface RenderResult {
    output?: string;
    diagnostics: Diagnostic[];
}

/** A generated AOT module. */
export interface GeneratedModule {
    path: string;
    code: string;
    dependencies: string[];
    origins: GeneratedOrigin[];
}

/** Generated line to template span mapping. */
export interface GeneratedOrigin {
    line: number;
    start: number;
    end: number;
}

/** Single-template AOT result. */
export interface PrecompileResult {
    module?: GeneratedModule;
    diagnostics: Diagnostic[];
    entryId: string;
}

/** Catalog AOT result. */
export interface PrecompileCatalogResult {
    modules: GeneratedModule[];
    manifest?: string;
    diagnostics: Diagnostic[];
    entryId: string;
}

/** Reference resolution result. */
export interface ResolveResult {
    id?: string;
    diagnostics: Diagnostic[];
}

/** Reference resolution request. */
export interface ResolveRequest {
    reference: string;
    from?: string;
    extensions?: string[];
    roots?: ResolveRoot[];
}

/** Reference resolution root. */
export interface ResolveRoot {
    name: string;
    scheme?: string;
    priority?: number;
    files: string[];
}

/** Parse request. */
export interface ParseRequest {
    source: string;
    file?: string;
    language?: string;
    delimiters?: Language;
}

/** Render IR request. */
export interface RenderIrRequest {
    ir: string;
    context: unknown;
    options?: RenderOptions;
    catalog?: CatalogSpec;
    /** Optional host registry; describes and implements custom entries. */
    registry?: CustomRegistry;
}

/** Render source request. */
export interface RenderSourceRequest {
    source: string;
    context: unknown;
    options?: RenderOptions;
    catalog?: CatalogSpec;
    /** Optional host registry; describes and implements custom entries. */
    registry?: CustomRegistry;
    delimiters?: Language;
}

/** Single-template precompile request. */
export interface PrecompileRequest {
    ir: string;
    options?: { strictUndefined?: boolean; entryId?: string; capabilities?: string[] };
}

/** Catalog precompile request. */
export interface PrecompileCatalogRequest {
    entryIr: string;
    catalog: CatalogSpec;
    options?: { strictUndefined?: boolean; entryId?: string; capabilities?: string[] };
}

/** Input accepted by {@link initWasm}. */
export interface InitOptions {
    /** Preloaded wasm bytes or module. Required in Node and preferred in Workers. */
    module?: BufferSource | WebAssembly.Module;
    /** URL or request to fetch the wasm from. Defaults to the module next to the glue. */
    url?: string | URL;
}

let ready = false;

/**
 * Instantiate the bridge. Call once before any capability.
 *
 * - `module` instantiates synchronously from bytes, which is the Node path.
 * - `url` fetches through the generated loader, which is the browser path.
 * - Neither falls back to the glue's default wasm location.
 */
export async function initWasm(options: InitOptions = {}): Promise<void> {
    if (ready) return;
    if (options.module) {
        glue.initSync({ module: options.module });
    } else if (options.url) {
        await glue.default({ module_or_path: options.url });
    } else {
        await glue.default();
    }
    ready = true;
}

/** Whether {@link initWasm} has completed. */
export function isWasmReady(): boolean {
    return ready;
}

/** Contract string reported by the loaded bridge. */
export function bindingContractVersion(): string {
    assertReady();
    return glue.bindingContractVersion();
}

/** Assert that the loaded bridge matches {@link BINDING_CONTRACT}. */
export function assertBindingContract(): void {
    const contract = bindingContractVersion();
    if (contract !== BINDING_CONTRACT) {
        throw new Error(`dejavu-wasm contract mismatch: expected ${BINDING_CONTRACT}, loaded ${contract}`);
    }
}

/** Parse template source into IR. */
export function parseIr(request: ParseRequest): IrResult {
    return parseJson(
        glue.parseIr(
            encode({
                source: request.source,
                file: request.file,
                language: request.language,
                delimiters: jsonOrUndefined(request.delimiters),
            }),
        ),
    );
}

/** Normalize IR JSON into its canonical shape. */
export function normalizeIr(ir: string): IrResult {
    return parseJson(glue.normalizeIr(encode({ ir })));
}

/** Render IR with an explicit context. */
export function renderIr(request: RenderIrRequest): RenderResult {
    return parseJson(
        glue.renderIr(
            encode({
                ir: request.ir,
                context: JSON.stringify(request.context),
                options: encodeRenderOptions(request.options),
                catalog: jsonOrUndefined(request.catalog),
            }),
            request.registry,
        ),
    );
}

/** Parse and render template source. */
export function renderSource(request: RenderSourceRequest): RenderResult {
    return parseJson(
        glue.renderSource(
            encode({
                source: request.source,
                context: JSON.stringify(request.context),
                options: encodeRenderOptions(request.options),
                catalog: jsonOrUndefined(request.catalog),
                delimiters: jsonOrUndefined(request.delimiters),
            }),
            request.registry,
        ),
    );
}

/** Resolve a template reference against catalog roots without reading files. */
export function resolveTemplateRef(request: ResolveRequest): ResolveResult {
    return parseJson(glue.resolveTemplateRef(encode(request)));
}

/** Precompile one template IR into an ESM module. */
export function precompileIrSingle(request: PrecompileRequest): PrecompileResult {
    return parseJson(glue.precompileIrSingle(encode({ ir: request.ir, options: jsonOrUndefined(request.options) })));
}

/** Precompile a whole catalog into ESM modules plus a manifest. */
export function precompileCatalog(request: PrecompileCatalogRequest): PrecompileCatalogResult {
    return parseJson(
        glue.precompileCatalog(
            encode({
                entryIr: request.entryIr,
                catalog: JSON.stringify(request.catalog),
                options: jsonOrUndefined(request.options),
            }),
        ),
    );
}

function assertReady(): void {
    if (!ready) throw new Error("dejavu-wasm is not initialized, call initWasm() before using the bridge");
}

function encode(request: unknown): string {
    assertReady();
    return JSON.stringify(request);
}

function parseJson<T>(raw: string): T {
    return JSON.parse(raw) as T;
}

function jsonOrUndefined(value: unknown): string | undefined {
    return value === undefined ? undefined : JSON.stringify(value);
}

/**
 * The render wire uses `strict` for the strict-undefined flag, while the
 * author-facing surface keeps the `strictUndefined` vocabulary shared with the
 * codegen options and the AOT runtime.
 */
function encodeRenderOptions(options: RenderOptions | undefined): string | undefined {
    if (options === undefined) return undefined;
    return JSON.stringify({
        strict: options.strictUndefined,
        name: options.name,
        capabilities: options.capabilities,
    });
}
