import {describe, expect, it} from "vitest";
import {DejavuEngine, markSafe} from "../src/index";
import {readFileSync, readdirSync} from "node:fs";
import {join} from "node:path";

const dokiLang = {
    syntaxMode: "template" as const,
    template: {
        codeStart: "{%",
        codeEnd: "%}",
        commentStart: "{#",
        commentEnd: "#}",
        supportFilterPipe: true,
        legacyFor: false,
    },
};

const themeDirs = [
    "E:/Spark 游戏引擎/doki-engine/examples/example-theme/templates",
    "E:/Spark 游戏引擎/doki-engine/packages/theme-template/templates",
    "E:/Spark 游戏引擎/doki-engine/examples/static-site/templates",
    "E:/Spark 游戏引擎/doki-engine/examples/docs-site/templates",
    "E:/Spark 游戏引擎/doki-engine/examples/blog-site/templates",
    "E:/Spark 游戏引擎/doki-engine/themes/default/templates",
    "E:/Spark 游戏引擎/doki-engine/examples/mvp-site/templates",
    "E:/Spark 游戏引擎/doki-engine/examples/init-site/templates",
];

describe("theme corpus parses on IR", () => {
    for (const dir of themeDirs) {
        it(dir.split("/").slice(-3).join("/"), () => {
            const eng = new DejavuEngine({language: dokiLang});
            for (const f of readdirSync(dir)) {
                if (!f.endsWith(".html")) continue;
                const src = readFileSync(join(dir, f), "utf8");
                eng.registerTemplate(f, src);
                eng.registerTemplate(f.replace(/\.html$/i, ""), src);
            }
            const ctx = {
                site: {title: "T", lang: "zh", author: "A", description: "D"},
                page: {
                    title: "P",
                    date: "2024-01-15",
                    author: "A",
                    tags: ["x"],
                    url: "/",
                    summary: "s",
                },
                pages: [
                    {
                        title: "P",
                        date: "2024-01-15",
                        author: "A",
                        tags: ["x"],
                        url: "/",
                        description: "e",
                        excerpt: "ex",
                    },
                ],
                content: markSafe("<p>hi</p>"),
                now: "2024-06-01T00:00:00.000Z",
            };
            let rendered = false;
            for (const name of ["index", "page", "post", "layout"]) {
                try {
                    const out = eng.renderTemplate(name, ctx);
                    expect(out.length).toBeGreaterThan(0);
                    rendered = true;
                    break;
                } catch (e) {
                    if (String(e).includes("not found")) continue;
                    throw e;
                }
            }
            expect(rendered).toBe(true);
        });
    }
});
