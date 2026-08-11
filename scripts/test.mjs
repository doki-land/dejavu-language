#!/usr/bin/env node
import { execSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = join(dirname(fileURLToPath(import.meta.url)), "..");
const rustManifest = join("projects", "dejavu.rs", "Cargo.toml");

/**
 * @param {string} cmd
 * @param {import("node:child_process").ExecSyncOptions} [options]
 */
function run(cmd, options = {}) {
    console.log(`$ ${cmd}`);
    execSync(cmd, {
        cwd: rootDir,
        stdio: "inherit",
        ...options,
    });
}

console.log("=== Dejavu test suite ===\n");

try {
    console.log("--- Rust ---");
    run(`cargo test --release --manifest-path ${rustManifest}`);

    console.log("\n--- TypeScript ---");
    run("pnpm --filter @dejavu/engine test");

    console.log("\n=== All tests passed ===");
} catch (error) {
    console.error("\n!!! Tests failed !!!");
    if (error instanceof Error) {
        console.error(error.message);
    }
    process.exit(1);
}
