import {describe, expect, it} from "vitest";
import {DejavuEngine, parseToIr} from "../src/index";

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

describe("parse language option (no mutable engine state)", () => {
    it("parse(source, { language }) uses delimiters without setLanguage", () => {
        const eng = new DejavuEngine(); // default <% %>
        const doc = eng.parse("{% title %}", {language: dokiLang});
        expect(doc.body.type).toBe("Template");
        if (doc.body.type === "Template") {
            expect(doc.body.children[0]?.type).toBe("Interpolation");
        }
        // engine default language unchanged — <% still works, {% without lang fails or treats as text
        const defaultDoc = eng.parse("<% title %>");
        expect(defaultDoc.body.type).toBe("Template");
    });

    it("renderSource(options.language) matches parse language", () => {
        const eng = new DejavuEngine();
        const out = eng.renderSource("{% name %}", {name: "X"}, {language: dokiLang});
        expect(out).toBe("X");
    });

    it("parseToIr accepts language directly", () => {
        const doc = parseToIr("{% a %}", {language: dokiLang});
        expect(doc.language.template.codeStart).toBe("{%");
    });
});
