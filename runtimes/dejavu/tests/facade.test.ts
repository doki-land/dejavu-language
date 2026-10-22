/**
 * Facade contract tests: every capability goes through the native engine.
 */

import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";

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
    tempDir = mkdtempSync(join(tmpdir(), "dejavu-facade-"));
});

afterAll(() => {
    delete process.env.DEJAVU_NATIVE_NODE;
});

describe("facade", () => {
    it("reports the binding contract", async () => {
        const { bindingContractVersion, BINDING_CONTRACT } = await import("../src/index");
        expect(bindingContractVersion()).toBe(BINDING_CONTRACT);
    });

    it("parses and renders without re-implementing the engine in TS", async () => {
        const { parse, renderSource, render } = await import("../src/index");
        const ir = parse("<% loop item in items %><% item %>;<% end loop %>", {
            file: "list.dejavu",
        });
        expect(render(ir, { items: ["a", "b"] })).toBe("a;b;");
        expect(renderSource("Hello, <% name %>!", { name: "Dejavu" })).toBe("Hello, Dejavu!");
    });

    it("normalizes IR deterministically", async () => {
        const { normalizeIr, parse } = await import("../src/index");
        const ir = parse("x", { file: "plain.dejavu" });
        expect(normalizeIr(ir)).toEqual(normalizeIr(JSON.stringify(ir)));
    });

    it("loads a catalog from a directory and renders includes through the engine", async () => {
        const { loadCatalogFromDirs, render, parse } = await import("../src/index");
        const dir = join(tempDir, "templates");
        mkdirSync(join(dir, "partials"), { recursive: true });
        writeFileSync(join(dir, "partials", "nav.dejavu"), "NAV");
        writeFileSync(join(dir, "index.dejavu"), "Hello, <% name %>!");

        const catalog = loadCatalogFromDirs([{ name: "root", scheme: "root", dir }]);
        expect(Object.keys(catalog.roots[0].files)).toEqual([
            "index.dejavu",
            "partials/nav.dejavu",
        ]);

        // The parser has no `include` statement yet, so the IR is built directly
        // from the engine's own document shape.
        const base = parse("x");
        const main = {
            irVersion: base.irVersion,
            language: base.language,
            body: {
                type: "Template",
                children: [
                    {
                        type: "Stmt.Include",
                        path: { type: "Expr.Literal", value: "root:partials/nav.dejavu" },
                        trim: "none",
                    },
                ],
            },
        };
        expect(render(main, {}, { catalog, name: "root:index.dejavu" })).toBe("NAV");
    });

    it("resolves path-v1 references without filesystem access", async () => {
        const { resolveTemplateRef } = await import("../src/index");
        const result = resolveTemplateRef({
            reference: "../nav",
            from: "pages:a/index.dejavu",
            roots: [{ name: "pages", files: ["nav.dejavu"] }],
        });
        expect(result.id).toBe("pages:nav.dejavu");
        expect(result.diagnostics).toEqual([]);
    });

    it("surfaces structured diagnostics as DejavuError", async () => {
        const { parse, DejavuError } = await import("../src/index");
        expect(() => parse("<% if %>", { file: "broken.dejavu" })).toThrow(DejavuError);
        try {
            parse("<% if %>", { file: "broken.dejavu" });
        } catch (error) {
            const dejavu = error as {
                code: string;
                stage: string;
                path?: string | null;
                span?: { start: number };
            };
            expect(dejavu.code).toMatch(/^dejavu\.parse\./);
            expect(dejavu.stage).toBe("parse");
            expect(dejavu.path).toBe("broken.dejavu");
            expect(dejavu.span?.start).toBeTypeOf("number");
        }
    });

    it("fails loudly when the native addon is missing", async () => {
        vi.resetModules();
        const previous = process.env.DEJAVU_NATIVE_NODE;
        process.env.DEJAVU_NATIVE_NODE = join(tempDir, "missing.node");
        try {
            const { loadNative } = await import("../src/index");
            expect(() => loadNative()).toThrow(/native addon missing/);
        } finally {
            process.env.DEJAVU_NATIVE_NODE = previous;
            vi.resetModules();
        }
    });
});
