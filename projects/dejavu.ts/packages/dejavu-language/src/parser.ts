import {
    DEFAULT_LANGUAGE,
    type IrDocument,
    type IrNode,
    type Language,
    type ParseOptions,
    type Trim,
} from "@dejavu/types";
import { ParseError } from "./error";
import { parseExpr } from "./expr";
import { lexCode } from "./lexer";
import type { CodeToken } from "./token";

type TrimMode = Trim;

/** Classify a code block from its token stream (no string-prefix hacks). */
type CodeHead =
    | { kind: "if"; testSlice: string; testAbs: number }
    | { kind: "loop"; item: string; iterSlice: string; iterAbs: number }
    | { kind: "else_if"; testSlice: string; testAbs: number }
    | { kind: "else" }
    | { kind: "end_if" }
    | { kind: "end_loop" }
    | { kind: "extends"; parentSlice: string; parentAbs: number }
    | { kind: "include"; pathSlice: string; pathAbs: number }
    | { kind: "block"; name: string }
    | { kind: "end_block" }
    | { kind: "super" }
    | { kind: "expr"; exprSlice: string; exprAbs: number };

function sliceFromTokens(
    content: string,
    contentBase: number,
    tokens: CodeToken[],
): { slice: string; abs: number } {
    if (tokens.length === 0) return { slice: "", abs: contentBase };
    const start = tokens[0]!.start;
    const end = tokens.at(-1)!.end;
    return { slice: content.slice(start, end), abs: contentBase + start };
}

function classifyCode(content: string, contentBase: number, tokens: CodeToken[]): CodeHead {
    if (tokens.length === 0) {
        return { kind: "expr", exprSlice: "", exprAbs: contentBase };
    }
    const t0 = tokens[0]!;
    if (t0.kind === "Ident" && t0.text === "if") {
        const { slice, abs } = sliceFromTokens(content, contentBase, tokens.slice(1));
        return { kind: "if", testSlice: slice, testAbs: abs };
    }
    if (t0.kind === "Ident" && t0.text === "loop") {
        if (tokens[1]?.kind !== "Ident") {
            throw new ParseError("loop requires item identifier", {
                start: contentBase + t0.start,
                length: Math.max(1, (tokens.at(-1)?.end ?? t0.end) - t0.start),
            });
        }
        if (tokens[2]?.kind !== "In") {
            throw new ParseError("loop requires `in`", {
                start: contentBase + t0.start,
                length: Math.max(1, (tokens.at(-1)?.end ?? t0.end) - t0.start),
                label: "expected `item in iterable`",
            });
        }
        const item = tokens[1]!.text;
        const { slice, abs } = sliceFromTokens(content, contentBase, tokens.slice(3));
        return { kind: "loop", item, iterSlice: slice, iterAbs: abs };
    }
    if (t0.kind === "Ident" && t0.text === "extends") {
        const { slice, abs } = sliceFromTokens(content, contentBase, tokens.slice(1));
        if (!slice) {
            throw new ParseError("extends requires a parent template expression", {
                start: contentBase + t0.start,
                length: Math.max(1, t0.end - t0.start),
            });
        }
        return { kind: "extends", parentSlice: slice, parentAbs: abs };
    }
    if (t0.kind === "Ident" && t0.text === "include") {
        const { slice, abs } = sliceFromTokens(content, contentBase, tokens.slice(1));
        if (!slice) {
            throw new ParseError("include requires a path expression", {
                start: contentBase + t0.start,
                length: Math.max(1, t0.end - t0.start),
            });
        }
        return { kind: "include", pathSlice: slice, pathAbs: abs };
    }
    if (t0.kind === "Ident" && t0.text === "block") {
        if (tokens[1]?.kind !== "Ident") {
            throw new ParseError("block requires a name", {
                start: contentBase + t0.start,
                length: Math.max(1, (tokens.at(-1)?.end ?? t0.end) - t0.start),
            });
        }
        if (tokens.length !== 2) {
            throw new ParseError("unexpected tokens after block name", {
                start: contentBase + tokens[2]!.start,
                length: 1,
            });
        }
        return { kind: "block", name: tokens[1]!.text };
    }
    if (t0.kind === "Ident" && t0.text === "super" && tokens.length === 1) {
        return { kind: "super" };
    }
    if (t0.kind === "Ident" && t0.text === "else") {
        if (tokens[1]?.kind === "Ident" && tokens[1].text === "if") {
            const { slice, abs } = sliceFromTokens(content, contentBase, tokens.slice(2));
            return { kind: "else_if", testSlice: slice, testAbs: abs };
        }
        if (tokens.length === 1) return { kind: "else" };
        throw new ParseError("unexpected tokens after `else`", {
            start: contentBase + tokens[1]!.start,
            length: 1,
        });
    }
    if (t0.kind === "Ident" && t0.text === "end") {
        if (tokens[1]?.kind === "Ident" && tokens[1].text === "if" && tokens.length === 2) {
            return { kind: "end_if" };
        }
        if (tokens[1]?.kind === "Ident" && tokens[1].text === "loop" && tokens.length === 2) {
            return { kind: "end_loop" };
        }
        if (tokens[1]?.kind === "Ident" && tokens[1].text === "block" && tokens.length === 2) {
            return { kind: "end_block" };
        }
        throw new ParseError("expected `end if`, `end loop`, or `end block`", {
            start: contentBase + t0.start,
            length: Math.max(1, (tokens.at(-1)?.end ?? t0.end) - t0.start),
        });
    }
    const { slice, abs } = sliceFromTokens(content, contentBase, tokens);
    return { kind: "expr", exprSlice: slice, exprAbs: abs };
}

/** Parse template source into Dejavu IR (T1 + inheritance surface). */
export function parseToIr(source: string, options: ParseOptions | string = {}): IrDocument {
    const opts: ParseOptions = typeof options === "string" ? { file: options } : (options ?? {});
    const file = opts.file ?? "template.dejavu";
    const language = structuredClone(opts.language ?? DEFAULT_LANGUAGE);
    return {
        irVersion: "1.0",
        language,
        body: {
            type: "Template",
            children: new TemplateParser(source, file, language).parse(),
        },
    };
}

class TemplateParser {
    private readonly codeStart: string;
    private readonly codeEnd: string;
    private readonly commentStart: string;
    private readonly commentEnd: string;

    constructor(
        private readonly source: string,
        private readonly file: string,
        language: Language,
    ) {
        this.codeStart = language.template.codeStart;
        this.codeEnd = language.template.codeEnd;
        this.commentStart = language.template.commentStart;
        this.commentEnd = language.template.commentEnd;
    }

    parse(): IrNode[] {
        const [children] = this.parseBody(0, []);
        return children;
    }

    private parseBody(i: number, stop: CodeHead["kind"][]): [IrNode[], number] {
        const children: IrNode[] = [];
        const source = this.source;
        const cs = this.codeStart;

        while (i < source.length) {
            if (source.startsWith(cs, i) && stop.length > 0) {
                const block = this.readCodeBlock(i);
                const head = classifyCode(block.content, block.contentBase, block.tokens);
                if (stop.includes(head.kind)) return [children, i];
            }

            if (source.startsWith(this.commentStart, i)) {
                const end = this.findDelimiter(i + this.commentStart.length, this.commentEnd);
                if (end < 0) {
                    throw new ParseError("unclosed comment", {
                        file: this.file,
                        start: i,
                        length: this.commentStart.length,
                        label: "comment starts here",
                    });
                }
                children.push({
                    type: "Comment",
                    value: source.slice(i + this.commentStart.length, end),
                });
                i = end + this.commentEnd.length;
                continue;
            }

            // Escape: codeStart + "!" → literal codeStart (e.g. `<%!` → `<%`)
            if (source.startsWith(cs + "!", i)) {
                children.push({ type: "Text", value: cs });
                i += cs.length + 1;
                continue;
            }

            if (source.startsWith(cs, i)) {
                const open = i;
                const block = this.readCodeBlock(i);
                i = block.next;
                const head = classifyCode(block.content, block.contentBase, block.tokens);

                if (stop.length > 0 && stop.includes(head.kind)) {
                    return [children, open];
                }

                if (head.kind === "if") {
                    const [node, ni] = this.parseIf(i, head, block.trim);
                    children.push(node);
                    i = ni;
                } else if (head.kind === "loop") {
                    const [node, ni] = this.parseLoop(i, head, block.trim);
                    children.push(node);
                    i = ni;
                } else if (head.kind === "block") {
                    const [node, ni] = this.parseBlock(i, head, block.trim);
                    children.push(node);
                    i = ni;
                } else if (head.kind === "extends") {
                    children.push({
                        type: "Stmt.Extends",
                        parent: parseExpr(head.parentSlice, {
                            source,
                            file: this.file,
                            base: head.parentAbs,
                        }),
                        trim: block.trim,
                    });
                } else if (head.kind === "include") {
                    children.push({
                        type: "Stmt.Include",
                        path: parseExpr(head.pathSlice, {
                            source,
                            file: this.file,
                            base: head.pathAbs,
                        }),
                        trim: block.trim,
                    });
                } else if (head.kind === "super") {
                    children.push({ type: "Stmt.Super", trim: block.trim });
                } else if (
                    head.kind === "end_if" ||
                    head.kind === "end_loop" ||
                    head.kind === "end_block" ||
                    head.kind === "else" ||
                    head.kind === "else_if"
                ) {
                    if (stop.length === 0) {
                        throw new ParseError(`unexpected control \`${head.kind}\``, {
                            file: this.file,
                            start: block.contentBase,
                            length: 1,
                        });
                    }
                    return [children, open];
                } else {
                    children.push({
                        type: "Interpolation",
                        expression: parseExpr(head.exprSlice, {
                            source,
                            file: this.file,
                            base: head.exprAbs,
                        }),
                        trim: block.trim,
                    });
                }
                continue;
            }

            const next = this.nextMarkup(i);
            if (next === null) {
                children.push({ type: "Text", value: source.slice(i) });
                break;
            }
            if (next > i) {
                children.push({ type: "Text", value: source.slice(i, next) });
                i = next;
            } else {
                i += 1;
            }
        }

        return [children, i];
    }

    private parseIf(
        i: number,
        head: Extract<CodeHead, { kind: "if" }>,
        trim: TrimMode,
    ): [IrNode, number] {
        const test = parseExpr(head.testSlice, {
            source: this.source,
            file: this.file,
            base: head.testAbs,
        });
        let [consequent, pos] = this.parseBody(i, ["else_if", "else", "end_if"]);
        i = pos;
        const elseIfs: IrNode[] = [];
        let alternate: IrNode[] | undefined;

        for (;;) {
            const block = this.readCodeBlock(i);
            const h = classifyCode(block.content, block.contentBase, block.tokens);
            if (h.kind === "else_if") {
                i = block.next;
                const t = parseExpr(h.testSlice, {
                    source: this.source,
                    file: this.file,
                    base: h.testAbs,
                });
                const [body, n] = this.parseBody(i, ["else_if", "else", "end_if"]);
                elseIfs.push({ type: "Stmt.ElseIf", test: t, consequent: body, trim: "none" });
                i = n;
            } else if (h.kind === "else") {
                i = block.next;
                const [body, n] = this.parseBody(i, ["end_if"]);
                alternate = body;
                i = n;
                const end = this.readCodeBlock(i);
                if (classifyCode(end.content, end.contentBase, end.tokens).kind !== "end_if") {
                    throw new ParseError("expected `end if`", {
                        file: this.file,
                        start: end.contentBase,
                        length: 1,
                    });
                }
                i = end.next;
                break;
            } else if (h.kind === "end_if") {
                i = block.next;
                break;
            } else {
                throw new ParseError(`expected if closer, got \`${h.kind}\``, {
                    file: this.file,
                    start: block.contentBase,
                    length: 1,
                });
            }
        }

        return [
            {
                type: "Stmt.If",
                test,
                consequent,
                elseIfs,
                ...(alternate ? { alternate } : {}),
                trim,
            },
            i,
        ];
    }

    private parseLoop(
        i: number,
        head: Extract<CodeHead, { kind: "loop" }>,
        trim: TrimMode,
    ): [IrNode, number] {
        const iterable = parseExpr(head.iterSlice, {
            source: this.source,
            file: this.file,
            base: head.iterAbs,
        });
        const [body, pos] = this.parseBody(i, ["end_loop"]);
        i = pos;
        const end = this.readCodeBlock(i);
        if (classifyCode(end.content, end.contentBase, end.tokens).kind !== "end_loop") {
            throw new ParseError("expected `end loop`", {
                file: this.file,
                start: end.contentBase,
                length: 1,
            });
        }
        return [{ type: "Stmt.For", item: head.item, iterable, body, trim }, end.next];
    }

    private parseBlock(
        i: number,
        head: Extract<CodeHead, { kind: "block" }>,
        trim: TrimMode,
    ): [IrNode, number] {
        const [body, pos] = this.parseBody(i, ["end_block"]);
        i = pos;
        const end = this.readCodeBlock(i);
        if (classifyCode(end.content, end.contentBase, end.tokens).kind !== "end_block") {
            throw new ParseError("expected `end block`", {
                file: this.file,
                start: end.contentBase,
                length: 1,
            });
        }
        return [{ type: "Stmt.Block", name: head.name, body, trim }, end.next];
    }

    private readCodeBlock(i: number): {
        tokens: CodeToken[];
        content: string;
        contentBase: number;
        trim: TrimMode;
        next: number;
    } {
        const source = this.source;
        const cs = this.codeStart;
        const ce = this.codeEnd;
        if (!source.startsWith(cs, i)) {
            throw new ParseError(`expected code open \`${cs}\``, {
                file: this.file,
                start: i,
                length: 1,
            });
        }
        let j = i + cs.length;
        let trim: TrimMode = "none";
        const mod = source[j];
        if (mod === "." || mod === "_" || mod === "-" || mod === "~" || mod === "=") {
            trim =
                mod === "_"
                    ? "ws"
                    : mod === "-"
                      ? "nl"
                      : mod === "~"
                        ? "ws_nl"
                        : mod === "="
                          ? "all"
                          : "none";
            j++;
        }
        const end = this.findDelimiter(j, ce);
        if (end < 0) {
            throw new ParseError("unclosed code block", {
                file: this.file,
                start: i,
                length: cs.length,
                label: "opens here",
            });
        }
        let contentBase = j;
        while (contentBase < end) {
            const c = source.charCodeAt(contentBase);
            if (c === 0x20 || c === 0x09 || c === 0x0a || c === 0x0d) contentBase++;
            else break;
        }
        let contentEnd = end;
        while (contentEnd > contentBase) {
            const c = source.charCodeAt(contentEnd - 1);
            if (c === 0x20 || c === 0x09 || c === 0x0a || c === 0x0d) contentEnd--;
            else break;
        }
        const content = source.slice(contentBase, contentEnd);
        const tokens = lexCode(content, {
            source,
            file: this.file,
            base: contentBase,
        }).filter((t) => t.kind !== "CodeEnd");
        return { tokens, content, contentBase, trim, next: end + ce.length };
    }

    private findDelimiter(from: number, delim: string): number {
        const source = this.source;
        const d0 = delim.charCodeAt(0);
        for (let i = from; i + delim.length <= source.length; i++) {
            if (source.charCodeAt(i) !== d0) continue;
            let ok = true;
            for (let k = 1; k < delim.length; k++) {
                if (source.charCodeAt(i + k) !== delim.charCodeAt(k)) {
                    ok = false;
                    break;
                }
            }
            if (ok) return i;
        }
        return -1;
    }

    private nextMarkup(from: number): number | null {
        const source = this.source;
        const markers = [this.codeStart, this.commentStart];
        let best: number | null = null;
        for (const m of markers) {
            const idx = source.indexOf(m, from);
            if (idx >= 0 && (best === null || idx < best)) best = idx;
        }
        return best;
    }
}

export { parseExpr } from "./expr";
export { ParseError } from "./error";
export { lexCode } from "./lexer";
export type { CodeToken, CodeTokenKind } from "./token";
