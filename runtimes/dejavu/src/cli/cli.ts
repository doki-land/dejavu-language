/**
 * Dejavu CLI command tree.
 *
 * Product command semantics live here: argv shape, config, stdio and exit codes.
 * `@vmz/commander` owns command context, help, diagnostic routing and exit semantics.
 * Each action only composes one or more N-API engine capabilities.
 */

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { createCli } from "@vmz/commander";

import { bindingContractVersion } from "../index.js";
import { runCheck } from "./check-cmd.js";
import { createDiagnosticReporter } from "./diagnostic.js";
import { runParse } from "./parse-cmd.js";
import { runRender } from "./render-cmd.js";

/** Build the CLI tree. Exported so tests can inspect help without spawning. */
export function createDejavuCli() {
    const reporter = createDiagnosticReporter();
    const cli = createCli("dejavu")
        .locales(resolveLocalesRoot(), { envKeys: ["DEJAVU_LOCALE", "LOCALE", "LANG", "LC_ALL"] })
        .onDiagnostic(reporter.handler)
        .intro("cli.intro");

    cli.command("parse", "cli.cmd.parse")
        .option("--out <file>", "cli.opt.out")
        .option("--language <id>", "cli.opt.language")
        .action((options, file) =>
            runParse(requirePositional(file, "parse"), {
                ...(typeof options.out === "string" ? { out: options.out } : {}),
                ...(typeof options.language === "string" ? { language: options.language } : {}),
            }),
        );

    cli.command("render", "cli.cmd.render")
        .option("--from-ir", "cli.opt.from-ir")
        .option("--ctx <file>", "cli.opt.ctx")
        .option("--strict", "cli.opt.strict")
        .option("--root <dir>", "cli.opt.root")
        .action((options, file) =>
            runRender(requirePositional(file, "render"), {
                fromIr: options["from-ir"] === true,
                ...(typeof options.ctx === "string" ? { ctx: options.ctx } : {}),
                strict: options.strict === true,
                ...(typeof options.root === "string" ? { root: options.root } : {}),
            }),
        );

    cli.command("check", "cli.cmd.check")
        .option("--extensions <list>", "cli.opt.extensions")
        .option("--root <dir>", "cli.opt.root")
        .action((options, ...paths) =>
            runCheck(paths, {
                ...(typeof options.extensions === "string"
                    ? { extensions: options.extensions }
                    : {}),
                ...(typeof options.root === "string" ? { root: options.root } : {}),
            }),
        );

    cli.command("version", "cli.cmd.version").action(() => {
        const meta = readPackageMeta();
        process.stdout.write(`${meta.name} ${meta.version} (${bindingContractVersion()})\n`);
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

/** Locales root: `dist/cli` → package root → `locales`. */
export function resolveLocalesRoot(): string {
    return resolve(dirname(fileURLToPath(import.meta.url)), "..", "..", "locales");
}

/** Read the package manifest from the facade root. */
export function readPackageMeta(): { name: string; version: string } {
    const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
    return JSON.parse(readFileSync(resolve(root, "package.json"), "utf8")) as {
        name: string;
        version: string;
    };
}

function requirePositional(value: string | undefined, command: string): string {
    if (!value) {
        throw new Error(`dejavu ${command}: missing required argument <file>`);
    }
    return value;
}
