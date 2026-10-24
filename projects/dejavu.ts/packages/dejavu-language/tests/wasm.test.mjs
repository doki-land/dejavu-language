import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import * as dejavu from "../dist/index.js";

const glueUrl = import.meta.resolve("@doki-land/dejavu-unknown-wasm32");
const wasmPath = join(dirname(fileURLToPath(glueUrl)), "dejavu_wasm_bg.wasm");
const wasmBytes = readFileSync(wasmPath);

test("capabilities reject calls before initWasm", () => {
    assert.equal(dejavu.isWasmReady(), false);
    assert.throws(() => dejavu.parseIr({ source: "<p>hi</p>" }), /initWasm/);
});

test("initWasm exposes the bridge contract", async () => {
    await dejavu.initWasm({ module: wasmBytes });
    assert.equal(dejavu.isWasmReady(), true);
    assert.equal(dejavu.bindingContractVersion(), dejavu.BINDING_CONTRACT);
    dejavu.assertBindingContract();
});

test("parseIr and renderSource handle statements and loops", () => {
    const source = "<% if flag %>on<% end if %><% loop item in items %>[<% item %>]<% end loop %>";
    const parsed = dejavu.parseIr({ source, file: "main.dejavu", delimiters: undefined });
    assert.equal(parsed.diagnostics.length, 0);
    assert.ok(parsed.ir);

    const rendered = dejavu.renderSource({ source, context: { flag: true, items: ["a", "b"] } });
    assert.equal(rendered.diagnostics.length, 0);
    assert.equal(rendered.output, "on[a][b]");

    const fromIr = dejavu.renderIr({ ir: parsed.ir, context: { flag: false, items: ["x"] } });
    assert.equal(fromIr.output, "[x]");
});

test("strictUndefined returns structured diagnostics", () => {
    const rendered = dejavu.renderSource({
        source: "<p><% missing %></p>",
        context: {},
        options: { strictUndefined: true },
    });
    assert.equal(rendered.output, undefined);
    assert.equal(rendered.diagnostics.length, 1);
    const [diagnostic] = rendered.diagnostics;
    assert.equal(diagnostic.severity, "error");
    assert.equal(diagnostic.code, "dejavu.template.undefined-variable");
    assert.equal(diagnostic.stage, "render");
    assert.match(diagnostic.argsJson, /missing/);
});

test("parse errors carry a span", () => {
    const parsed = dejavu.parseIr({ source: "<% if %>" });
    assert.equal(parsed.ir, undefined);
    assert.equal(parsed.diagnostics.length, 1);
    assert.equal(parsed.diagnostics[0].severity, "error");
    assert.ok(parsed.diagnostics[0].span);
});

test("resolveTemplateRef resolves a canonical id", () => {
    const resolved = dejavu.resolveTemplateRef({
        reference: "partial",
        roots: [{ name: "project", files: ["partial.dejavu"] }],
    });
    assert.equal(resolved.diagnostics.length, 0);
    assert.equal(resolved.id, "project:partial.dejavu");
});

test("renderSource loads includes from an explicit catalog", () => {
    const rendered = dejavu.renderSource({
        source: 'A-<% include "partial.dejavu" %>-Z',
        context: { name: "X" },
        catalog: {
            roots: [{ name: "project", files: { "partial.dejavu": JSON.stringify(parseIrOk("partial:<% name %>")) } }],
        },
    });
    assert.deepEqual(rendered.diagnostics, []);
    assert.equal(rendered.output, "A-partial:X-Z");
});

test("precompileIrSingle returns a module with origins", () => {
    const parsed = dejavu.parseIr({ source: "<p><% name %></p>" });
    assert.ok(parsed.ir);
    const result = dejavu.precompileIrSingle({ ir: parsed.ir });
    assert.deepEqual(result.diagnostics, []);
    assert.ok(result.module);
    assert.equal(result.module.path, "index.js");
    assert.match(result.module.code, /export/);
    assert.ok(result.module.origins.length > 0);
});

test("precompileCatalog returns modules and a manifest", () => {
    const entry = parseIrOk('A-<% include "partial.dejavu" %>');
    const partial = parseIrOk("partial:<% name %>");
    const result = dejavu.precompileCatalog({
        entryIr: JSON.stringify(entry),
        catalog: {
            roots: [{ name: "project", files: { "main.dejavu": JSON.stringify(entry), "partial.dejavu": JSON.stringify(partial) } }],
        },
    });
    assert.deepEqual(result.diagnostics, []);
    assert.ok(result.modules.length >= 2);
    assert.ok(result.manifest);
});

function parseIrOk(source) {
    const parsed = dejavu.parseIr({ source });
    assert.deepEqual(parsed.diagnostics, []);
    return JSON.parse(parsed.ir);
}
