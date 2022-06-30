import { describe, expect, it } from "vitest";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { DejavuEngine, type Language } from "../src";

const root = join(
    dirname(fileURLToPath(import.meta.url)),
    "../../../../specifications/conformance/inheritance",
);

describe("specifications/conformance/inheritance", () => {
    const cases = readdirSync(root, { withFileTypes: true })
        .filter((d) => d.isDirectory())
        .map((d) => d.name)
        .sort();

    for (const name of cases) {
        it(name, () => {
            const dir = join(root, name);
            const templatesDir = join(dir, "templates");
            const entry = readFileSync(join(dir, "entry.txt"), "utf8").trim();
            const ctx = JSON.parse(readFileSync(join(dir, "context.ctx.json"), "utf8"));
            const expectedOut = readFileSync(join(dir, "expected.out.txt"), "utf8");

            let language: Language | undefined;
            const languagePath = join(dir, "language.json");
            if (existsSync(languagePath)) {
                language = JSON.parse(readFileSync(languagePath, "utf8")) as Language;
            }

            const eng = new DejavuEngine(language ? { language } : undefined);
            for (const file of readdirSync(templatesDir)) {
                const source = readFileSync(join(templatesDir, file), "utf8");
                eng.registerTemplate(file, source);
            }

            expect(eng.renderTemplate(entry, ctx)).toBe(expectedOut);
        });
    }
});
