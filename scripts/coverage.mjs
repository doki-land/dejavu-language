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
function runCommand(cmd, options = {}) {
    console.log(`$ ${cmd}`);
    return execSync(cmd, {
        cwd: rootDir,
        stdio: "inherit",
        ...options,
    });
}

console.log("=== 开始运行覆盖率测试 ===\n");

try {
    console.log("--- 运行 TypeScript 覆盖率测试 ---");
    runCommand("pnpm --filter @dejavu/engine test -- --coverage");

    console.log("\n--- 运行 Rust 覆盖率测试 ---");
    runCommand(
        `cargo tarpaulin --out Html --output-dir ./projects/dejavu.rs/target/coverage --manifest-path ${rustManifest}`,
    );

    console.log("\n=== 覆盖率测试完成 ===");
    console.log("TypeScript 覆盖率报告: projects/dejavu.ts/packages/*/coverage");
    console.log("Rust 覆盖率报告: projects/dejavu.rs/target/coverage/tarpaulin-report.html");
} catch (error) {
    console.error("\n!!! 覆盖率测试失败 !!!");
    console.error(error);
    process.exit(1);
}
