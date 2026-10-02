import { Dejavu, type IrDocument, type IrValue } from "dejavu";

export type DejavuContext = Record<string, IrValue>;

/** Render IR → HTML body string (cross-language identical path). */
export function htmlFromIr(ir: IrDocument | string, ctx: DejavuContext = {}): string {
    return Dejavu.render(ir, ctx);
}

/** Parse source → IR → HTML. Prefer `htmlFromIr` when IR is already available. */
export function htmlFromSource(source: string, ctx: DejavuContext = {}): string {
    return Dejavu.renderSource(source, ctx);
}

/**
 * Hono helper: returns a `Response` with `text/html`.
 * Package id uses the host prefix `hono-dejavu`; core API remains `dejavu`.
 */
export function dejavuHtml(body: string, init?: ResponseInit): Response {
    const headers = new Headers(init?.headers);
    if (!headers.has("content-type")) {
        headers.set("content-type", "text/html; charset=utf-8");
    }
    return new Response(body, { ...init, headers });
}

export function dejavuHtmlFromIr(
    ir: IrDocument | string,
    ctx: DejavuContext = {},
    init?: ResponseInit,
): Response {
    return dejavuHtml(htmlFromIr(ir, ctx), init);
}

export function dejavuHtmlFromSource(
    source: string,
    ctx: DejavuContext = {},
    init?: ResponseInit,
): Response {
    return dejavuHtml(htmlFromSource(source, ctx), init);
}

export { Dejavu };
