/**
 * `dejavu render <file>` — render template source or IR.
 *
 * Engine work comes from the N-API `renderSource` / `renderIr` capabilities.
 */

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

import { loadNative } from "../native.js";
import { createDiagnosticReporter } from "./diagnostic.js";
import { templateId } from "./template-id.js";
import type { TemplateContext } from "../types.js";

export type RenderCommandOptions = {
    fromIr?: boolean;
    ctx?: string;
    strict?: boolean;
    root?: string;
};

export function runRender(file: string, options: RenderCommandOptions = {}): number {
    const source = readFileSync(file, "utf8");
    const context = readContext(options.ctx);
    const engine = loadNative();
    const reporter = createDiagnosticReporter();
    const request = {
        context: JSON.stringify(context),
        options: JSON.stringify({
            strict: options.strict === true,
            name: templateId(file, options.root ?? dirname(resolve(file))),
        }),
    };

    const response = options.fromIr
        ? engine.renderIr({ ir: source, ...request })
        : engine.renderSource({ source, ...request });

    reporter.report(response.diagnostics);
    if (reporter.hasErrors(response.diagnostics)) return 1;
    process.stdout.write(`${response.output ?? ""}\n`);
    return 0;
}

function readContext(file?: string): TemplateContext {
    if (!file) return {};
    const raw = readFileSync(file, "utf8").trim();
    if (raw === "") return {};
    return JSON.parse(raw) as TemplateContext;
}
