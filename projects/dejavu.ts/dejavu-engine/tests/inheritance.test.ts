import {describe, expect, it} from "vitest";
import {DejavuEngine, MapTemplateLoader, parseToIr, renderIr} from "../src/index";

describe("T1 still works without loader", () => {
    it("renders interpolation", () => {
        const doc = parseToIr("Hello, <% name %>!");
        expect(renderIr(doc, {name: "World"})).toBe("Hello, World!");
    });
});

describe("inheritance with loader", () => {
    it("extends + block override", () => {
        const loader = MapTemplateLoader.fromRecord({
            "base.dejavu": `<html><% block body %>base<% end block %></html>`,
            "child.dejavu": `<% extends "base.dejavu" %><% block body %>child<% end block %>`,
        });
        const out = renderIr(loader.load("child.dejavu"), {}, {loader, name: "child.dejavu"});
        expect(out).toBe("<html>child</html>");
    });

    it("super inserts parent block body", () => {
        const loader = MapTemplateLoader.fromRecord({
            "base.dejavu": `<% block body %>BASE<% end block %>`,
            "child.dejavu": `<% extends "base.dejavu" %><% block body %><% super %>+CHILD<% end block %>`,
        });
        const out = renderIr(loader.load("child.dejavu"), {}, {loader, name: "child.dejavu"});
        expect(out).toBe("BASE+CHILD");
    });

    it("include embeds another template", () => {
        const loader = MapTemplateLoader.fromRecord({
            "partial.dejavu": `P:<% name %>`,
            "main.dejavu": `A-<% include "partial.dejavu" %>-Z`,
        });
        const out = renderIr(
            loader.load("main.dejavu"),
            {name: "X"},
            {loader, name: "main.dejavu"},
        );
        expect(out).toBe("A-P:X-Z");
    });

    it("DejavuEngine.registerTemplate + renderTemplate", () => {
        const eng = new DejavuEngine();
        eng.registerTemplate(
            "base.dejavu",
            `layout[<% block content %>default<% end block %>]`,
        );
        eng.registerTemplate(
            "page.dejavu",
            `<% extends "base.dejavu" %><% block content %><% title %><% end block %>`,
        );
        expect(eng.renderTemplate("page.dejavu", {title: "Hi"})).toBe("layout[Hi]");
    });

    it("throws without loader when extends present", () => {
        const doc = parseToIr(`<% extends "base.dejavu" %><% block body %>x<% end block %>`);
        expect(() => renderIr(doc, {})).toThrow(/template loader/);
    });

    it("supports custom delimiters for Doki-style {% %}", () => {
        const language = {
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
        const eng = new DejavuEngine({language});
        eng.registerTemplate("base.html", `<b>{% block body %}B{% end block %}</b>`);
        eng.registerTemplate(
            "page.html",
            `{% extends "base.html" %}{% block body %}{% title %}{% end block %}`,
        );
        expect(eng.renderTemplate("page.html", {title: "OK"})).toBe("<b>OK</b>");
    });
});
