import { ParseError } from "./error";
import type { CodeToken, CodeTokenKind } from "./token";

/**
 * Hand-written lexer for code inside `<% ... %>`.
 * Character-class scanning belongs here — not in the parser.
 */
export function lexCode(
    input: string,
    opts?: { source?: string; file?: string; base?: number },
): CodeToken[] {
    const source = opts?.source ?? input;
    const file = opts?.file ?? "template.dejavu";
    const base = opts?.base ?? 0;
    const tokens: CodeToken[] = [];
    let i = 0;
    const n = input.length;

    const push = (
        kind: CodeTokenKind,
        start: number,
        end: number,
        text = "",
        value?: boolean | null,
    ) => {
        tokens.push({ kind, text, value, start, end });
    };

    const fail = (start: number, length: number, message: string) => {
        throw new ParseError(message, {
            file,
            start: base + start,
            length,
            label: "bad token",
        });
    };

    while (i < n) {
        const c = input.charCodeAt(i);
        // whitespace
        if (c === 0x20 || c === 0x09 || c === 0x0a || c === 0x0d) {
            i++;
            continue;
        }

        // two-char ops
        if (i + 1 < n) {
            const two = input.slice(i, i + 2);
            const map2: Record<string, CodeTokenKind> = {
                "%>": "CodeEnd",
                "|>": "PipeOp",
                "||": "OrOr",
                "&&": "AndAnd",
                "==": "EqEq",
                "!=": "NotEq",
                "<=": "LessEq",
                ">=": "GreaterEq",
            };
            if (two in map2) {
                push(map2[two]!, i, i + 2, two);
                i += 2;
                continue;
            }
        }

        const one = input[i]!;
        const map1: Record<string, CodeTokenKind> = {
            "<": "Less",
            ">": "Greater",
            "+": "Plus",
            "-": "Minus",
            "*": "Star",
            "/": "Slash",
            "%": "Percent",
            "!": "Bang",
            ".": "Dot",
            ",": "Comma",
            ":": "Colon",
            "(": "LParen",
            ")": "RParen",
            "[": "LBracket",
            "]": "RBracket",
        };
        if (one in map1) {
            push(map1[one]!, i, i + 1, one);
            i++;
            continue;
        }

        // string
        if (one === '"' || one === "'") {
            const quote = one;
            const start = i;
            i++;
            let text = "";
            while (i < n) {
                const ch = input[i]!;
                if (ch === "\\") {
                    if (i + 1 >= n) fail(start, i - start + 1, "unterminated string escape");
                    text += input[i + 1]!;
                    i += 2;
                    continue;
                }
                if (ch === quote) {
                    i++;
                    push("String", start, i, text);
                    break;
                }
                text += ch;
                i++;
            }
            if (tokens.at(-1)?.kind !== "String" || tokens.at(-1)?.start !== start) {
                fail(start, Math.max(1, i - start), "unterminated string");
            }
            continue;
        }

        // number
        if (c >= 0x30 && c <= 0x39) {
            const start = i;
            i++;
            while (i < n) {
                const d = input.charCodeAt(i);
                if (d >= 0x30 && d <= 0x39) i++;
                else break;
            }
            if (input[i] === ".") {
                i++;
                while (i < n) {
                    const d = input.charCodeAt(i);
                    if (d >= 0x30 && d <= 0x39) i++;
                    else break;
                }
            }
            push("Number", start, i, input.slice(start, i));
            continue;
        }

        // ident / keywords
        if (isIdentStart(c)) {
            const start = i;
            i++;
            while (i < n && isIdentContinue(input.charCodeAt(i))) i++;
            const text = input.slice(start, i);
            if (text === "true") push("Bool", start, i, text, true);
            else if (text === "false") push("Bool", start, i, text, false);
            else if (text === "null") push("Null", start, i, text, null);
            else if (text === "in") push("In", start, i, text);
            else push("Ident", start, i, text);
            continue;
        }

        fail(i, 1, `invalid token in expression (${JSON.stringify(one)})`);
    }

    void source;
    return tokens;
}

function isIdentStart(c: number): boolean {
    return (c >= 0x41 && c <= 0x5a) || (c >= 0x61 && c <= 0x7a) || c === 0x5f;
}

function isIdentContinue(c: number): boolean {
    return isIdentStart(c) || (c >= 0x30 && c <= 0x39);
}
