import { describe, expect, it } from "vitest";
import { parseToIr, renderIr, markSafe } from "../src/index";

describe("safe / filters / pipe", () => {
    it("auto-escapes interpolation by default", () => {
        const doc = parseToIr("<% html %>");
        expect(renderIr(doc, { html: "<b>x</b>" })).toBe("&lt;b&gt;x&lt;/b&gt;");
    });

    it("safe / raw skip escape", () => {
        const doc = parseToIr("<% html |> safe %>");
        expect(renderIr(doc, { html: "<b>x</b>" })).toBe("<b>x</b>");
        const doc2 = parseToIr("<% html |> raw %>");
        expect(renderIr(doc2, { html: "<i>y</i>" })).toBe("<i>y</i>");
    });

    it("markSafe host values skip escape", () => {
        const doc = parseToIr("<% content %>");
        expect(renderIr(doc, { content: markSafe("<p>ok</p>") })).toBe("<p>ok</p>");
    });

    it("pipe is |> only; supports filter: arg", () => {
        const doc = parseToIr("{% name |> upper %}", {
            language: {
                syntaxMode: "template",
                template: {
                    codeStart: "{%",
                    codeEnd: "%}",
                    commentStart: "{#",
                    commentEnd: "#}",
                    supportFilterPipe: true,
                    legacyFor: false,
                },
            },
        });
        expect(renderIr(doc, { name: "hi" })).toBe("HI");

        const dated = parseToIr("<% d |> date: '%Y-%m-%d' %>");
        expect(renderIr(dated, { d: "2024-01-15T12:00:00.000Z" })).toMatch(/^\d{4}-\d{2}-\d{2}$/);

        expect(() => parseToIr("<% name | upper %>")).toThrow();
    });

    it("escape filter forces escaping", () => {
        const doc = parseToIr("<% html |> escape %>");
        expect(renderIr(doc, { html: "<a>" })).toBe("&lt;a&gt;");
    });

    it("strictUndefined throws on missing id", () => {
        const doc = parseToIr("<% missing %>");
        expect(() => renderIr(doc, {}, { strictUndefined: true })).toThrow(/undefined variable/);
        expect(renderIr(doc, {})).toBe("");
    });

    it("default with colon args", () => {
        const doc = parseToIr("<% title |> default: 'N/A' %>");
        expect(renderIr(doc, {})).toBe("N/A");
        expect(renderIr(doc, { title: "Hi" })).toBe("Hi");
    });
});
