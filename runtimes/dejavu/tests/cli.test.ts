/**
 * CLI contract tests: command semantics live in TS, engine work comes from N-API.
 */

import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";

const ROOT = resolve(fileURLToPath(new URL(".", import.meta.url)), "..", "..", "..");
const NATIVE = {
    "linux-x64": "linux-x64/dejavu.linux-x64-gnu.node",
    "linux-arm64": "linux-arm64/dejavu.linux-arm64-gnu.node",
    "darwin-x64": "darwin-x64/dejavu.darwin-x64.node",
    "darwin-arm64": "darwin-arm64/dejavu.darwin-arm64.node",
    "win32-x64": "win32-x64/dejavu.win32-x64-msvc.node",
    "win32-arm64": "win32-arm64/dejavu.win32-arm64-msvc.node",
} as const;

let tempDir = "";

beforeAll(() => {
    const key = `${process.platform}-${process.arch}` as keyof typeof NATIVE;
    process.env.DEJAVU_NATIVE_NODE ??= join(ROOT, "compilers", "dejavu", "npm", NATIVE[key]);
    tempDir = mkdtempSync(join(tmpdir(), "dejavu-cli-"));
});

afterEach(() => {
    vi.restoreAllMocks();
});

function capture() {
    const out: string[] = [];
    const err: string[] = [];
    vi.spyOn(process.stdout, "write").mockImplementation((chunk: string | Uint8Array) => {
        out.push(String(chunk));
        return true;
    });
    vi.spyOn(process.stderr, "write").mockImplementation((chunk: string | Uint8Array) => {
        err.push(String(chunk));
        return true;
    });
    return { stdout: () => out.join(""), stderr: () => err.join("") };
}

describe("cli", () => {
    it("parses a template to IR JSON", async () => {
        const { runCli } = await import("../src/cli/cli");
        const file = join(tempDir, "hello.dejavu");
        writeFileSync(file, "Hello, <% name %>!");
        const io = capture();

        expect(await runCli(["parse", file])).toBe(0);
        const ir = JSON.parse(io.stdout());
        expect(ir.irVersion).toBeTypeOf("string");
        expect(io.stderr()).toBe("");
    });

    it("renders a template with a context file", async () => {
        const { runCli } = await import("../src/cli/cli");
        const file = join(tempDir, "greet.dejavu");
        const ctx = join(tempDir, "ctx.json");
        writeFileSync(file, "Hello, <% name %>!");
        writeFileSync(ctx, JSON.stringify({ name: "Dejavu" }));
        const io = capture();

        expect(await runCli(["render", file, "--ctx", ctx])).toBe(0);
        expect(io.stdout().trim()).toBe("Hello, Dejavu!");
    });

    it("exits 1 with a stable code for a broken template", async () => {
        const { runCli } = await import("../src/cli/cli");
        const file = join(tempDir, "broken.dejavu");
        writeFileSync(file, "<% if %>");
        const io = capture();

        expect(await runCli(["check", file])).toBe(1);
        expect(io.stderr()).toMatch(/dejavu\.parse\.[a-z-]+/);
    });

    it("reports the binding contract", async () => {
        const { runCli } = await import("../src/cli/cli");
        const io = capture();

        expect(await runCli(["version"])).toBe(0);
        expect(io.stdout()).toContain("dejavu.napi/1");
    });

    it("rejects unknown commands with exit code 2", async () => {
        const { runCli } = await import("../src/cli/cli");
        const io = capture();

        expect(await runCli(["nope"])).toBe(2);
        expect(io.stderr()).toContain("unknown command");
    });
});
