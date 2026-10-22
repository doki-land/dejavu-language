/**
 * Diagnostic presentation for the Dejavu CLI.
 *
 * The engine emits language-neutral diagnostics (`code` + structured args + span).
 * Translation uses the product locale catalog; formatting stays here.
 */

import type { CommandDiagnostic, DiagnosticContext, DiagnosticHandler } from "@vmz/commander";

import type { DejavuDiagnostic } from "../types.js";

/** Reporter shared by command actions and `onDiagnostic`. */
export type DiagnosticReporter = {
    /** Framework hook for thrown command errors. */
    handler: DiagnosticHandler;
    /** Emit one or more engine diagnostics to stderr. */
    report: (diagnostics: DejavuDiagnostic[], context?: DiagnosticContext) => void;
    /** True when any diagnostic is error severity. */
    hasErrors: (diagnostics: DejavuDiagnostic[]) => boolean;
};

export function createDiagnosticReporter(
    write: (text: string) => void = (text) => process.stderr.write(text),
): DiagnosticReporter {
    return {
        handler: (diagnostic, context) => {
            write(`${format(diagnostic, context)}\n`);
        },
        report: (diagnostics, context) => {
            for (const diagnostic of diagnostics) {
                write(`${format(toCommandDiagnostic(diagnostic), context)}\n`);
            }
        },
        hasErrors: (diagnostics) => diagnostics.some((d) => d.severity === "error"),
    };
}

function toCommandDiagnostic(diagnostic: DejavuDiagnostic): CommandDiagnostic {
    return {
        severity: diagnostic.severity,
        code: diagnostic.code,
        args: stringArgs(diagnostic.argsJson),
        ...(diagnostic.path ? { path: diagnostic.path } : {}),
        ...(diagnostic.span ? { span: diagnostic.span } : {}),
        message: diagnostic.message,
    };
}

function format(diagnostic: CommandDiagnostic, context?: DiagnosticContext): string {
    const code = diagnostic.code ?? "dejavu.unknown";
    const text = translate(diagnostic, context);
    const location = diagnostic.path ? `${diagnostic.path}:` : "";
    const span = diagnostic.span ? `${diagnostic.span.start}-${diagnostic.span.end}` : "";
    const where = location || span ? ` [${location}${span}]` : "";
    return `${diagnostic.severity ?? "error"} ${code}${where}: ${text}`;
}

function translate(diagnostic: CommandDiagnostic, context?: DiagnosticContext): string {
    const args = diagnostic.args ?? {};
    if (context) {
        const translated = context.t(diagnostic.code ?? "", args);
        if (translated !== `{{${diagnostic.code}}}`) return translated;
    }
    return diagnostic.message ?? "";
}

function stringArgs(argsJson: string): Record<string, string> {
    try {
        const value = JSON.parse(argsJson) as unknown;
        if (!value || typeof value !== "object" || Array.isArray(value)) return {};
        const out: Record<string, string> = {};
        for (const [key, item] of Object.entries(value as Record<string, unknown>)) {
            out[key] = typeof item === "string" ? item : JSON.stringify(item);
        }
        return out;
    } catch {
        return {};
    }
}
