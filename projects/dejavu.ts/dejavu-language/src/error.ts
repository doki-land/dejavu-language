/** Span-aware parse failure. */
export class ParseError extends Error {
    readonly file: string;
    readonly start: number;
    readonly length: number;
    readonly label: string;

    constructor(
        message: string,
        opts: { file?: string; start: number; length?: number; label?: string },
    ) {
        super(message);
        this.name = "ParseError";
        this.file = opts.file ?? "template.dejavu";
        this.start = opts.start;
        this.length = Math.max(1, opts.length ?? 1);
        this.label = opts.label ?? "here";
    }
}
