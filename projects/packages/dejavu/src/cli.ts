/**
 * Dejavu CLI command tree.
 *
 * Command semantics (argv shape, exit codes, stdio) live here.
 * Each action only composes one or more facade-level capabilities
 * from `./index.js` — never touches N-API directly.
 *
 * Exit codes: 0 = success, 1 = diagnostic error / runtime failure,
 * 2 = invalid argv (handled by commander itself).
 */

import { readFileSync, writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

import { createCli } from "@vmz/commander";

import {
    parse,
    precompile,
    bindingContractVersion,
    DejavuError,
} from "./index.js";
import type { Diagnostic } from "./index.js";

/** Build the CLI tree. Exported so tests can inspect help without spawning. */
export function createDejavuCli() {
    const cli = createCli("dejavu");

    cli.locales(resolveLocalesRoot(), { envKeys: ["DEJAVU_LOCALE", "LOCALE", "LANG", "LC_ALL"] });
    cli.intro("cli.intro");

    cli.command("version", "cli.cmd.version").action(() => {
        const meta = readPackageMeta();
        process.stdout.write(
            `${meta.name} ${meta.version} (${bindingContractVersion()})\n`,
        );
        return 0;
    });

    cli.command("check", "cli.cmd.check").action((options, file) => {
        const source = readSource(requirePositional(file, "check"));
        const diagnostics = checkTemplate(source);
        return reportDiagnostics(diagnostics);
    });

    cli.command("compile", "cli.cmd.compile")
        .option("--out <file>", "cli.opt.out")
        .action((options, file) => {
            const input = requirePositional(file, "compile");
            const output = typeof options.out === "string" ? options.out : `${input}.js`;
            return runCompile(input, output);
        });

    // TODO(B5): replace stub with full dejavu.config.ts-driven generate.
    cli.command("generate", "cli.cmd.generate").action(() => {
        process.stderr.write(
            "dejavu generate: not yet implemented (B5). Use `dejavu compile <file>` for single-template AOT.\n",
        );
        return 0;
    });

    return cli;
}

/** Parse argv and return the process exit code. */
export async function runCli(argv: string[] = process.argv.slice(2)): Promise<number> {
    try {
        return await createDejavuCli().parse(argv);
    } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        process.stderr.write(`${message}\n`);
        return 1;
    }
}

// ---------------------------------------------------------------------------
// Internal command runners
// ---------------------------------------------------------------------------

function checkTemplate(source: string): Diagnostic[] {
    try {
        parse(source);
        return [];
    } catch (error) {
        if (error instanceof DejavuError) {
            return error.diagnostics;
        }
        return [
            {
                code: "dejavu.cli.unknown",
                severity: "error",
                stage: "host",
                message: error instanceof Error ? error.message : String(error),
                argsJson: "{}",
                related: [],
            },
        ];
    }
}

function reportDiagnostics(diagnostics: Diagnostic[]): number {
    if (diagnostics.length === 0) {
        process.stdout.write("ok\n");
        return 0;
    }
    let hasErrors = false;
    for (const d of diagnostics) {
        const prefix = d.severity.toUpperCase();
        const where = d.path ? ` (${d.path}${d.span ? `:${d.span.start}` : ""})` : "";
        process.stderr.write(
            `${prefix} ${d.code}${where}: ${d.message}\n`,
        );
        if (d.severity === "error") hasErrors = true;
    }
    return hasErrors ? 1 : 0;
}

function runCompile(input: string, output: string): number {
    let source: string;
    try {
        source = readSource(input);
    } catch (error) {
        process.stderr.write(
            `dejavu compile: cannot read ${input}: ${error instanceof Error ? error.message : String(error)}\n`,
        );
        return 1;
    }

    try {
        const result = precompile(source, { file: input, entryId: input });

        for (const d of result.diagnostics) {
            if (d.severity === "error") continue;
            const prefix = d.severity.toUpperCase();
            process.stderr.write(`${prefix} ${d.code}: ${d.message}\n`);
        }

        if (result.modules.length === 0) {
            process.stderr.write("dejavu compile: no module produced\n");
            return 1;
        }

        const module = result.modules[0]!;
        try {
            writeFileSync(output, module.code, "utf8");
            const errors = result.diagnostics.filter((d) => d.severity === "error").length;
            const warnings = result.diagnostics.filter((d) => d.severity === "warning").length;
            process.stdout.write(
                `wrote ${output} (${errors} errors, ${warnings} warnings)\n`,
            );
            return errors > 0 ? 1 : 0;
        } catch (error) {
            process.stderr.write(
                `dejavu compile: cannot write ${output}: ${error instanceof Error ? error.message : String(error)}\n`,
            );
            return 1;
        }
    } catch (error) {
        if (error instanceof DejavuError) {
            return reportDiagnostics(error.diagnostics);
        }
        process.stderr.write(
            `dejavu compile: ${error instanceof Error ? error.message : String(error)}\n`,
        );
        return 1;
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function readSource(file: string): string {
    const abs = resolve(file);
    return readFileSync(abs, "utf8");
}

function readPackageMeta(): { name: string; version: string } {
    const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
    return JSON.parse(
        readFileSync(resolve(root, "package.json"), "utf8"),
    ) as { name: string; version: string };
}

/** Locales root: `dist/cli.js` → package root → `locales/`. */
export function resolveLocalesRoot(): string {
    return resolve(dirname(fileURLToPath(import.meta.url)), "..", "locales");
}

function requirePositional(value: string | undefined, command: string): string {
    if (!value) {
        throw new Error(`dejavu ${command}: missing required argument <file>`);
    }
    return value;
}

// ---------------------------------------------------------------------------
// Self-invocation when executed directly
// ---------------------------------------------------------------------------

const invokedDirectly = (() => {
    try {
        const entry = process.argv[1];
        if (!entry) return false;
        return fileURLToPath(import.meta.url) === resolve(entry);
    } catch {
        return false;
    }
})();

if (invokedDirectly) {
    runCli().then((code) => {
        process.exit(code);
    }).catch((err) => {
        const msg = err instanceof Error ? err.message : String(err);
        process.stderr.write(`${msg}\n`);
        process.exit(1);
    });
}
