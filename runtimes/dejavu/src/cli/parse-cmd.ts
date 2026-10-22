/**
 * `dejavu parse <file>` — parse template source into IR.
 *
 * Composes one engine capability (`parseIr`); no engine semantics are re-implemented.
 */

import { readFileSync, writeFileSync } from "node:fs";

import { DejavuError } from "../errors.js";
import { loadNative } from "../native.js";
import { createDiagnosticReporter } from "./diagnostic.js";

export type ParseCommandOptions = {
    out?: string;
    language?: string;
};

export function runParse(file: string, options: ParseCommandOptions = {}): number {
    const source = readFileSync(file, "utf8");
    const response = loadNative().parseIr({
        source,
        file,
        ...(options.language === undefined ? {} : { language: options.language }),
    });

    const reporter = createDiagnosticReporter();
    reporter.report(response.diagnostics);
    if (reporter.hasErrors(response.diagnostics)) return 1;
    if (typeof response.ir !== "string") {
        throw new DejavuError({
            code: "dejavu.host.missing-ir",
            severity: "error",
            stage: "host",
            message: "parseIr returned no IR and no error diagnostic",
            argsJson: "{}",
            related: [],
        });
    }

    if (options.out) {
        writeFileSync(options.out, `${response.ir}\n`);
    } else {
        process.stdout.write(`${response.ir}\n`);
    }
    return 0;
}
