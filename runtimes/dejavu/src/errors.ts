/**
 * Error raised when the engine reports an error-severity diagnostic.
 *
 * The shape is intentionally diagnostic-friendly: `@vmz/commander` routes thrown
 * errors through `onDiagnostic`, and downstream hosts can inspect `code` / `args`.
 */

import type { DejavuDiagnostic, DejavuSpan } from "./types.js";

/** Parse `argsJson` once, tolerating malformed payloads. */
function parseArgs(argsJson: string): Record<string, unknown> {
    try {
        const value = JSON.parse(argsJson) as unknown;
        if (value && typeof value === "object" && !Array.isArray(value)) {
            return value as Record<string, unknown>;
        }
    } catch {
        // fall through
    }
    return {};
}

export class DejavuError extends Error {
    readonly code: string;
    readonly severity: string;
    readonly stage: string;
    readonly path?: string | null;
    readonly templateId?: string | null;
    readonly span?: DejavuSpan | null;
    readonly args: Record<string, unknown>;
    readonly diagnostics: DejavuDiagnostic[];

    constructor(diagnostic: DejavuDiagnostic, diagnostics: DejavuDiagnostic[] = [diagnostic]) {
        super(diagnostic.message || diagnostic.code);
        this.name = "DejavuError";
        this.code = diagnostic.code;
        this.severity = diagnostic.severity;
        this.stage = diagnostic.stage;
        this.path = diagnostic.path;
        this.templateId = diagnostic.templateId;
        this.span = diagnostic.span;
        this.args = parseArgs(diagnostic.argsJson);
        this.diagnostics = diagnostics;
    }

    /** Build from an engine response, returning the first error diagnostic. */
    static fromDiagnostics(diagnostics: DejavuDiagnostic[]): DejavuError | undefined {
        const errors = diagnostics.filter((d) => d.severity === "error");
        const first = errors[0];
        return first ? new DejavuError(first, diagnostics) : undefined;
    }
}
