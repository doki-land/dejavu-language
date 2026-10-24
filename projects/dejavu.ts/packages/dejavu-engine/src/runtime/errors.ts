/**
 * Structured runtime error for generated AOT modules.
 *
 * Generated code never throws a bare `Error`: every failure carries the stable
 * diagnostic `code`, the `render` stage and a human-readable message so hosts
 * can classify errors without parsing `message`.
 */

export class DejavuRuntimeError extends Error {
    readonly code: string;
    readonly stage = "render";

    constructor(code: string, detail?: string) {
        super(detail ? `${code}: ${detail}` : code);
        this.name = "DejavuRuntimeError";
        this.code = code;
    }
}

/** Build a structured render error. */
export function runtimeError(code: string, detail?: string): DejavuRuntimeError {
    return new DejavuRuntimeError(code, detail);
}
