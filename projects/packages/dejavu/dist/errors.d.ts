/**
 * Error raised when the engine reports an error-severity diagnostic.
 *
 * The shape is intentionally diagnostic-friendly: CLI layers (via
 * `@vmz/commander`) route thrown errors through `onDiagnostic`, and
 * downstream hosts can inspect `code` / `args` / `diagnostics`.
 */
import type { Diagnostic, Span } from "./types.js";
export declare class DejavuError extends Error {
    readonly code: string;
    readonly severity: string;
    readonly stage: string;
    readonly path?: string;
    readonly templateId?: string;
    readonly span?: Span;
    readonly args: Record<string, unknown>;
    readonly diagnostics: Diagnostic[];
    constructor(diagnostic: Diagnostic, diagnostics?: Diagnostic[]);
    /** Build from an engine response, returning the first error diagnostic. */
    static fromDiagnostics(diagnostics: Diagnostic[]): DejavuError | undefined;
}
