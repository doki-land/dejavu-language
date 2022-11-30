import {describe, expect, it} from "vitest";
import {
    DejavuEngine,
    PathTemplateLoader,
    renderIr,
} from "../src/index";

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

describe("PathTemplateLoader", () => {
    it("resolves bare name via extensions and prefers higher-priority root", () => {
        const loader = new PathTemplateLoader({
            language: dokiLang,
            roots: [
                {name: "theme", priority: 10, scheme: "theme"},
                {name: "project", priority: 20},
            ],
        });
        loader.set("layout.html", "THEME", "theme");
        loader.set("layout.html", "PROJECT", "project");

        const hit = loader.resolve("layout");
        expect(hit.id).toBe("project:layout.html");
        expect(renderIr(hit.document, {}, {loader, name: hit.id})).toBe("PROJECT");

        const themeOnly = loader.resolve("theme:layout");
        expect(themeOnly.id).toBe("theme:layout.html");
    });

    it("resolves relative include against from's directory", () => {
        const loader = new PathTemplateLoader({
            language: dokiLang,
            roots: [{name: "project", priority: 1}],
        });
        loader.set(
            "pages/child.html",
            `{% include "./partials/head.html" %}BODY`,
        );
        loader.set("pages/partials/head.html", "HEAD-");

        expect(loader.resolve("./partials/head.html", {from: "project:pages/child.html"}).id).toBe(
            "project:pages/partials/head.html",
        );

        const eng = new DejavuEngine({language: dokiLang, loader});
        expect(eng.renderTemplate("pages/child.html")).toBe("HEAD-BODY");
    });

    it("rejects path escape and reports missing with from hint", () => {
        const loader = new PathTemplateLoader({
            language: dokiLang,
            roots: [{name: "project", priority: 1}],
        });
        loader.set("a.html", "A");
        expect(() =>
            loader.resolve("../../x.html", {from: "project:a.html"}),
        ).toThrow(/escapes root|not found/);
        expect(() => loader.resolve("missing", {from: "project:a.html"})).toThrow(
            /template not found: missing \(from project:a.html\)/,
        );
    });

    it("project override wins over theme for same bare name", () => {
        const loader = new PathTemplateLoader({
            language: dokiLang,
            roots: [
                {name: "theme", priority: 10, scheme: "theme"},
                {name: "project", priority: 20},
            ],
        });
        loader.set(
            "base.html",
            `{% block body %}T{% end block %}`,
            "theme",
        );
        loader.set(
            "page.html",
            `{% extends "base.html" %}{% block body %}P{% end block %}`,
            "project",
        );
        // theme also has base — project page extends bare base → project if present, else theme
        loader.set("base.html", `{% block body %}PBASE{% end block %}`, "project");

        const eng = new DejavuEngine({language: dokiLang, loader});
        expect(eng.renderTemplate("page")).toBe("P");
    });

    it("same name in different roots stays distinct via scheme", () => {
        const loader = new PathTemplateLoader({
            language: dokiLang,
            roots: [
                {name: "theme", priority: 10, scheme: "theme"},
                {name: "project", priority: 20},
            ],
        });
        loader.set("layout.html", "from-theme", "theme");
        loader.set("layout.html", "from-project", "project");

        const eng = new DejavuEngine({language: dokiLang, loader});
        eng.registerTemplate(
            "page.html",
            `{% extends "theme:layout.html" %}X`,
        );
        // child with only extends and no blocks — parent body
        expect(eng.renderTemplate("page.html")).toBe("from-theme");
    });
});
