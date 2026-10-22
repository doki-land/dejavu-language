/**
 * `dejavu check <path...>` — parse every template and report structured diagnostics.
 *
 * Reference/load failures surface during `render`; check keeps the parse contract
 * fast and side-effect free.
 */

import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

import { loadNative } from "../native.js";
import { createDiagnosticReporter } from "./diagnostic.js";
import { templateId } from "./template-id.js";
import type { DejavuDiagnostic } from "../types.js";

export type CheckCommandOptions = {
    extensions?: string;
    root?: string;
};

const DEFAULT_EXTENSIONS = [".dejavu", ".doki", ".html"];

export function runCheck(paths: string[], options: CheckCommandOptions = {}): number {
    const extensions = options.extensions
        ? options.extensions.split(",").map((ext) => (ext.startsWith(".") ? ext : `.${ext}`))
        : DEFAULT_EXTENSIONS;

    const files = paths.flatMap((path) => collect(path, extensions));
    if (files.length === 0) {
        process.stderr.write(
            `error dejavu.check.no-templates: no templates under ${paths.join(", ")}\n`,
        );
        return 2;
    }

    const reporter = createDiagnosticReporter();
    const diagnostics: DejavuDiagnostic[] = [];
    for (const file of files) {
        const response = loadNative().parseIr({
            source: readFileSync(file, "utf8"),
            file: templateId(file, options.root),
        });
        diagnostics.push(...response.diagnostics);
    }

    reporter.report(diagnostics);
    return reporter.hasErrors(diagnostics) ? 1 : 0;
}

function collect(path: string, extensions: string[]): string[] {
    const info = statSync(path);
    if (info.isFile()) return [path];
    if (!info.isDirectory()) return [];

    const out: string[] = [];
    for (const entry of readdirSync(path, { withFileTypes: true })) {
        const full = join(path, entry.name);
        if (entry.isDirectory()) {
            out.push(...collect(full, extensions));
            continue;
        }
        if (entry.isFile() && extensions.some((ext) => entry.name.endsWith(ext))) out.push(full);
    }
    return out.sort();
}
