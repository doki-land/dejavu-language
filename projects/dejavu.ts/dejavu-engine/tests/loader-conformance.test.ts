import {describe, expect, it} from "vitest";
import {existsSync, readFileSync, readdirSync, statSync} from "node:fs";
import {join, dirname, relative} from "node:path";
import {fileURLToPath} from "node:url";
import {
    CatalogTemplateLoader,
    TemplateLoaderError,
    type TemplateRootConfig,
} from "../src/index";

const root = join(
    dirname(fileURLToPath(import.meta.url)),
    "../../../../specifications/conformance/loader",
);

function walkFiles(dir: string, base = dir): string[] {
    const out: string[] = [];
    for (const name of readdirSync(dir)) {
        const full = join(dir, name);
        if (statSync(full).isDirectory()) out.push(...walkFiles(full, base));
        else out.push(relative(base, full).replace(/\\/g, "/"));
    }
    return out;
}

function loadCase(dir: string): CatalogTemplateLoader {
    const rootsJson = JSON.parse(
        readFileSync(join(dir, "roots.json"), "utf8"),
    ) as TemplateRootConfig[];
    const templatesRoot = join(dir, "templates");
    const roots: TemplateRootConfig[] = rootsJson.map((r) => {
        const files = new Map<string, string>();
        const rootDir = join(templatesRoot, r.name);
        if (existsSync(rootDir)) {
            for (const rel of walkFiles(rootDir)) {
                files.set(rel, readFileSync(join(rootDir, rel), "utf8"));
            }
        }
        return {...r, files};
    });
    return new CatalogTemplateLoader({
        roots,
        extensions: ["", ".html", ".doki", ".dejavu"],
        defaultRoot: roots.sort((a, b) => b.priority - a.priority)[0]!.name,
    });
}

describe("specifications/conformance/loader", () => {
    const cases = readdirSync(root, {withFileTypes: true})
        .filter((d) => d.isDirectory())
        .map((d) => d.name)
        .sort();

    for (const name of cases) {
        it(name, () => {
            const dir = join(root, name);
            const loader = loadCase(dir);
            const entry = readFileSync(join(dir, "entry.txt"), "utf8").trim();
            const fromPath = join(dir, "from.txt");
            const from = existsSync(fromPath)
                ? readFileSync(fromPath, "utf8").trim()
                : undefined;

            const errPath = join(dir, "expected.error.json");
            if (existsSync(errPath)) {
                const expected = JSON.parse(readFileSync(errPath, "utf8")) as {
                    code: string;
                    ref?: string;
                    from?: string;
                };
                try {
                    loader.resolve(entry, from ? {from} : undefined);
                    expect.fail("expected TemplateLoaderError");
                } catch (e) {
                    expect(e).toBeInstanceOf(TemplateLoaderError);
                    const diag = (e as TemplateLoaderError).diagnostic;
                    expect(diag.code).toBe(expected.code);
                    if (expected.ref) expect(diag.ref).toBe(expected.ref);
                    if (expected.from) expect(diag.from).toBe(expected.from);
                }
                return;
            }

            const expectedId = readFileSync(join(dir, "expected.id.txt"), "utf8").trim();
            const hit = loader.resolve(entry, from ? {from} : undefined);
            expect(hit.id).toBe(expectedId);
        });
    }
});
