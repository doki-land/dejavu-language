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
/** Build the CLI tree. Exported so tests can inspect help without spawning. */
export declare function createDejavuCli(): import("@vmz/commander").Cli;
/** Parse argv and return the process exit code. */
export declare function runCli(argv?: string[]): Promise<number>;
/** Locales root: `dist/cli.js` → package root → `locales/`. */
export declare function resolveLocalesRoot(): string;
