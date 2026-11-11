import assert from "node:assert/strict";
import { test } from "node:test";

import { renderTemplate, templateCapabilities } from "../dist/index.js";

test("templateCapabilities() reports the dejavu engine", () => {
    const capabilities = templateCapabilities();
    assert.equal(capabilities.length, 1);
    const [capability] = capabilities;
    assert.equal(capability.id, "dejavu");
    assert.equal(capability.status, "ready");
    assert.equal(capability.content, "html");
    assert.equal(capability.contract, "dejavu/v0");
    assert.ok(capability.extensions.includes(".dejavu"), "dejavu extension is claimed");
});

test("renderTemplate() renders through the shared envelope", () => {
    const result = renderTemplate({
        engine: "dejavu",
        source: "<% name %>",
        data: { name: "Ada" },
    });
    assert.equal(result.text, "Ada");
    assert.equal(result.content, "html");
    assert.deepEqual(result.dependencies, []);
    assert.deepEqual(result.diagnostics, []);
});

test("renderTemplate() reports an unknown engine instead of throwing", () => {
    const result = renderTemplate({ engine: "hbs", source: "x", data: {} });
    assert.equal(result.text, undefined);
    assert.equal(result.diagnostics[0]?.code, "template.unknown-engine");
});
