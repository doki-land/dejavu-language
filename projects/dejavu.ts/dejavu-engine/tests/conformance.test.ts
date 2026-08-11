import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { compareIrJson, engine, normalizeIrJson, renderIr, type IrDocument } from "../src";

const root = join(
    dirname(fileURLToPath(import.meta.url)),
    "../../../../specifications/conformance/t1",
);

describe("specifications/conformance/t1", () => {
    const cases = readdirSync(root, { withFileTypes: true })
        .filter((d) => d.isDirectory())
        .map((d) => d.name)
        .sort();

    for (const name of cases) {
        it(name, () => {
            const dir = join(root, name);
            const input = readFileSync(join(dir, "input.dejavu"), "utf8");
            const expectedIr = readFileSync(join(dir, "expected.ir.json"), "utf8");
            const ctx = JSON.parse(readFileSync(join(dir, "context.ctx.json"), "utf8"));
            const expectedOut = readFileSync(join(dir, "expected.out.txt"), "utf8");

            const doc = engine.parse(input);
            expect(
                compareIrJson(JSON.stringify(doc), expectedIr),
                `IR mismatch\n${JSON.stringify(normalizeIrJson(JSON.stringify(doc)), null, 2)}`,
            ).toBe(true);

            const out = renderIr(JSON.parse(expectedIr) as IrDocument, ctx);
            expect(out).toBe(expectedOut);
        });
    }
});
