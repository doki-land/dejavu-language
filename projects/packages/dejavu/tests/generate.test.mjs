import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { buildGeneratePlan, writeOutput } from "../dist/generate.js";

const BASE64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/** Decode one Base64 VLQ segment into its delta fields. */
function decodeSegment(segment) {
    const fields = [];
    let index = 0;
    while (index < segment.length) {
        let result = 0;
        let shift = 0;
        let digit;
        do {
            digit = BASE64.indexOf(segment[index++]);
            result += (digit & 31) << shift;
            shift += 5;
        } while (digit & 32);
        const negative = result & 1;
        result >>= 1;
        fields.push(negative ? -result : result);
    }
    return fields;
}

/**
 * Reduce `mappings` to the first segment of every generated line.
 *
 * Codegen emits one origin per construct, so the first segment on a line is the
 * template position that line starts at.
 */
function firstMappingPerLine(mappings) {
    const positions = new Map();
    let source = 0;
    let line = 0;
    let column = 0;
    mappings.split(";").forEach((entry, generatedLine) => {
        if (!entry) return;
        const [generatedColumn, sourceDelta, lineDelta, columnDelta] = decodeSegment(entry.split(",")[0]);
        assert.equal(generatedColumn, 0, "generated lines must map from column 0");
        source += sourceDelta;
        line += lineDelta;
        column += columnDelta;
        positions.set(generatedLine, { source, line, column });
    });
    return positions;
}

/** Write a throwaway project with the given templates and a matching config. */
function scaffold(templates) {
    const dir = mkdtempSync(join(tmpdir(), "dejavu-generate-test-"));
    writeFileSync(
        join(dir, "package.json"),
        `${JSON.stringify({ name: "fixture", private: true, type: "module" }, null, 2)}\n`,
    );
    mkdirSync(join(dir, "templates"), { recursive: true });
    for (const [name, text] of Object.entries(templates)) {
        writeFileSync(join(dir, "templates", name), text);
    }
    writeFileSync(
        join(dir, "dejavu.config.ts"),
        [
            "export default {",
            '  roots: [{ name: "app", dir: "./templates" }],',
            '  generate: { outDir: "./out", entries: { home: "app:home.dejavu" } },',
            "};",
            "",
        ].join("\n"),
    );
    return dir;
}

test("source maps point generated lines at their template spans", async (t) => {
    const template = "<h1><% title %></h1>\n<% if ok %>\n<p><% ok %></p>\n<% end if %>\n";
    const dir = scaffold({ "home.dejavu": template });
    t.after(() => rmSync(dir, { recursive: true, force: true }));

    const plan = await buildGeneratePlan({ cwd: dir });
    writeOutput(plan);

    const code = readFileSync(join(dir, "out", "home.js"), "utf8");
    const map = JSON.parse(readFileSync(join(dir, "out", "home.js.map"), "utf8"));

    // Stable logical source ids: no machine paths, and the template text ships inline.
    assert.deepEqual(map.sources, ["app:home.dejavu"]);
    assert.deepEqual(map.sourcesContent, [template]);
    assert.equal(map.file, "home.js");

    const positions = firstMappingPerLine(map.mappings);
    const lines = code.split("\n");
    const lineOf = (needle) => {
        const index = lines.findIndex((line) => line.includes(needle));
        assert.notEqual(index, -1, `generated code has no line containing ${needle}`);
        return index;
    };
    const escapes = lines
        .map((line, index) => (line.includes("__out.push(htmlEscape(valueToString(__v") ? index : -1))
        .filter((index) => index !== -1);
    assert.equal(escapes.length, 2, "one interpolated push per interpolation");

    assert.deepEqual(positions.get(lineOf('__out.push("<h1>")')), { source: 0, line: 0, column: 0 });
    // An interpolation maps from the line that evaluates it, which is the line above its push.
    assert.deepEqual(positions.get(escapes[0] - 1), { source: 0, line: 0, column: 4 });
    assert.deepEqual(positions.get(lineOf("if (isTruthy(ctx.ok))")), { source: 0, line: 1, column: 0 });
    assert.deepEqual(positions.get(escapes[1] - 1), { source: 0, line: 2, column: 3 });
});

test("source maps stay byte-stable across regenerations", async (t) => {
    const dir = scaffold({ "home.dejavu": "<p><% value %></p>\n" });
    t.after(() => rmSync(dir, { recursive: true, force: true }));

    const first = await buildGeneratePlan({ cwd: dir });
    writeOutput(first);
    const map = readFileSync(join(dir, "out", "home.js.map"), "utf8");

    const second = await buildGeneratePlan({ cwd: dir });
    writeOutput(second);

    assert.equal(readFileSync(join(dir, "out", "home.js.map"), "utf8"), map);
    assert.notEqual(JSON.parse(map).mappings, "");
});
