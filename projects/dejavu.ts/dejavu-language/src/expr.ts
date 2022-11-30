import type {IrNode} from "@dejavu/types";
import {ParseError} from "./error";
import {lexCode} from "./lexer";
import type {CodeToken, CodeTokenKind} from "./token";

/** Pratt-style expression parser over a [`CodeToken`] stream. */
export class ExprParser {
    private readonly source: string;
    private readonly file: string;
    private readonly base: number;
    private readonly tokens: CodeToken[];
    private pos = 0;

    constructor(source: string, file: string, base: number, input: string) {
        this.source = source;
        this.file = file;
        this.base = base;
        this.tokens = lexCode(input, {source, file, base});
        for (const t of this.tokens) {
            if (t.kind === "CodeEnd") {
                throw new ParseError("unexpected `%>` inside expression", {
                    file,
                    start: base + t.start,
                    length: t.end - t.start,
                });
            }
        }
    }

    parse(): IrNode {
        const expr = this.parsePipe();
        if (this.pos !== this.tokens.length) {
            const span = this.peekSpan();
            throw new ParseError("trailing input in expression", {
                file: this.file,
                start: span.start,
                length: span.length,
                label: "unexpected",
            });
        }
        return expr;
    }

    private peek(): CodeToken | undefined {
        return this.tokens[this.pos];
    }

    private peekKind(): CodeTokenKind | undefined {
        return this.tokens[this.pos]?.kind;
    }

    private peekSpan(): { start: number; length: number } {
        const t = this.tokens[this.pos];
        if (t) return {start: this.base + t.start, length: Math.max(1, t.end - t.start)};
        const last = this.tokens.at(-1);
        const end = this.base + (last?.end ?? 0);
        return {start: end, length: 1};
    }

    private bump(): CodeToken | undefined {
        return this.tokens[this.pos++];
    }

    private expectIdent(): string {
        const t = this.bump();
        if (t?.kind === "Ident") return t.text;
        const span = this.peekSpan();
        throw new ParseError("expected identifier", {
            file: this.file,
            start: span.start,
            length: span.length,
            label: "expected ident",
        });
    }

    private parsePipe(): IrNode {
        let left = this.parseOr();
        while (this.peekKind() === "PipeOp") {
            this.bump();
            const filter = this.expectIdent();
            const args: IrNode[] = [];
            if (this.peekKind() === "LParen") {
                this.bump();
                if (this.peekKind() !== "RParen") {
                    for (; ;) {
                        args.push(this.parsePipe());
                        if (this.peekKind() === "Comma") {
                            this.bump();
                            continue;
                        }
                        break;
                    }
                }
                if (this.bump()?.kind !== "RParen") {
                    const span = this.peekSpan();
                    throw new ParseError("expected `)` after filter arguments", {
                        file: this.file,
                        start: span.start,
                        length: span.length,
                    });
                }
            } else if (this.peekKind() === "Colon") {
                // Liquid / Doki: `value | date: '%Y-%m-%d'` / `value | default: x`
                this.bump();
                args.push(this.parseOr());
                while (this.peekKind() === "Comma") {
                    this.bump();
                    args.push(this.parseOr());
                }
            }
            left = {type: "Expr.Pipe", expression: left, filter, arguments: args};
        }
        return left;
    }

    private parseOr(): IrNode {
        let left = this.parseAnd();
        while (this.peekKind() === "OrOr") {
            this.bump();
            left = {type: "Expr.Binary", operator: "||", left, right: this.parseAnd()};
        }
        return left;
    }

    private parseAnd(): IrNode {
        let left = this.parseCmp();
        while (this.peekKind() === "AndAnd") {
            this.bump();
            left = {type: "Expr.Binary", operator: "&&", left, right: this.parseCmp()};
        }
        return left;
    }

    private parseCmp(): IrNode {
        const left = this.parseAdd();
        const kind = this.peekKind();
        const op =
            kind === "EqEq"
                ? "=="
                : kind === "NotEq"
                    ? "!="
                    : kind === "LessEq"
                        ? "<="
                        : kind === "GreaterEq"
                            ? ">="
                            : kind === "Less"
                                ? "<"
                                : kind === "Greater"
                                    ? ">"
                                    : kind === "In"
                                        ? "in"
                                        : null;
        if (op) {
            this.bump();
            return {type: "Expr.Binary", operator: op, left, right: this.parseAdd()};
        }
        return left;
    }

    private parseAdd(): IrNode {
        let left = this.parseMul();
        for (; ;) {
            const kind = this.peekKind();
            if (kind === "Plus" || kind === "Minus") {
                const op = kind === "Plus" ? "+" : "-";
                this.bump();
                left = {type: "Expr.Binary", operator: op, left, right: this.parseMul()};
            } else break;
        }
        return left;
    }

    private parseMul(): IrNode {
        let left = this.parseUnary();
        for (; ;) {
            const kind = this.peekKind();
            if (kind === "Star" || kind === "Slash" || kind === "Percent") {
                const op = kind === "Star" ? "*" : kind === "Slash" ? "/" : "%";
                this.bump();
                left = {type: "Expr.Binary", operator: op, left, right: this.parseUnary()};
            } else break;
        }
        return left;
    }

    private parseUnary(): IrNode {
        const kind = this.peekKind();
        if (kind === "Bang" || kind === "Minus" || kind === "Plus") {
            const op = kind === "Bang" ? "!" : kind === "Minus" ? "-" : "+";
            this.bump();
            return {type: "Expr.Unary", operator: op, argument: this.parseUnary()};
        }
        return this.parsePostfix();
    }

    private parsePostfix(): IrNode {
        let left = this.parsePrimary();
        for (; ;) {
            const kind = this.peekKind();
            if (kind === "Dot") {
                this.bump();
                left = {type: "Expr.Member", object: left, property: this.expectIdent()};
            } else if (kind === "LBracket") {
                this.bump();
                const index = this.parsePipe();
                if (this.bump()?.kind !== "RBracket") {
                    const span = this.peekSpan();
                    throw new ParseError("expected `]`", {
                        file: this.file,
                        start: span.start,
                        length: span.length,
                    });
                }
                left = {type: "Expr.Index", object: left, index};
            } else if (kind === "LParen") {
                this.bump();
                const args: IrNode[] = [];
                if (this.peekKind() !== "RParen") {
                    for (; ;) {
                        args.push(this.parsePipe());
                        if (this.peekKind() === "Comma") {
                            this.bump();
                            continue;
                        }
                        break;
                    }
                }
                if (this.bump()?.kind !== "RParen") {
                    const span = this.peekSpan();
                    throw new ParseError("expected `)`", {
                        file: this.file,
                        start: span.start,
                        length: span.length,
                    });
                }
                left = {type: "Expr.Call", callee: left, arguments: args};
            } else break;
        }
        return left;
    }

    private parsePrimary(): IrNode {
        const t = this.bump();
        if (!t) {
            const span = this.peekSpan();
            throw new ParseError("unexpected end of expression", {
                file: this.file,
                start: span.start,
                length: span.length,
            });
        }
        switch (t.kind) {
            case "String":
                return {type: "Expr.Literal", value: t.text};
            case "Bool":
                return {type: "Expr.Literal", value: Boolean(t.value)};
            case "Null":
                return {type: "Expr.Literal", value: null};
            case "Number": {
                const num = Number(t.text);
                if (Number.isNaN(num)) {
                    throw new ParseError(`invalid number \`${t.text}\``, {
                        file: this.file,
                        start: this.base + t.start,
                        length: t.end - t.start,
                    });
                }
                return {type: "Expr.Literal", value: num};
            }
            case "Ident":
                return {type: "Expr.Identifier", name: t.text};
            case "LParen": {
                const e = this.parsePipe();
                if (this.bump()?.kind !== "RParen") {
                    const span = this.peekSpan();
                    throw new ParseError("expected `)`", {
                        file: this.file,
                        start: span.start,
                        length: span.length,
                    });
                }
                return e;
            }
            default: {
                throw new ParseError("unexpected token in expression", {
                    file: this.file,
                    start: this.base + t.start,
                    length: Math.max(1, t.end - t.start),
                });
            }
        }
    }
}

export function parseExpr(
    input: string,
    opts?: { source?: string; file?: string; base?: number },
): IrNode {
    const source = opts?.source ?? input;
    const file = opts?.file ?? "template.dejavu";
    const base = opts?.base ?? 0;
    return new ExprParser(source, file, base, input).parse();
}
