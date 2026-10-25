import assert from "node:assert/strict";
import { test } from "node:test";

import { DejavuError, parse, render, renderIr } from "../dist/index.js";

test("render() raises a structured DejavuError for parse errors", () => {
    assert.throws(
        () => render("<% if %>"),
        (error) => {
            assert.ok(error instanceof DejavuError, "error is a DejavuError");
            assert.equal(error.name, "DejavuError");
            assert.equal(error.severity, "error");
            assert.match(error.code, /^dejavu\.parse\./);
            assert.ok(Array.isArray(error.diagnostics), "diagnostics is an array");
            assert.ok(error.diagnostics.length > 0, "diagnostics is not empty");
            assert.ok(
                error.diagnostics.some((diagnostic) => diagnostic.severity === "error"),
                "diagnostics contains an error",
            );
            assert.equal(typeof error.span, "object", "parse error keeps its span");
            assert.equal(typeof error.span.start, "number");
            assert.equal(typeof error.span.end, "number");
            return true;
        },
    );
});

test("render() dispatches host registry entries", () => {
    const output = render(
        "<% greeting |> shout %>|<% greeting |> bolder %>|<% twice(21) %><% if n is evenish %>Y<% end if %>",
        { greeting: "hi", n: 4 },
        {
            custom: {
                entries: [
                    { namespace: "filter", name: "shout" },
                    { namespace: "filter", name: "bolder", capability: "safe-output" },
                    { namespace: "function", name: "twice", minArgs: 1, maxArgs: 1 },
                    { namespace: "test", name: "evenish" },
                ],
                filter: {
                    shout: (value) => `${value}!`,
                    bolder: (value) => `<b>${value}</b>`,
                },
                function: { twice: (args) => Number(args[0]) * 2 },
                test: { evenish: (value) => Number(value) % 2 === 0 },
            },
        },
    );
    assert.equal(output, "hi!|<b>hi</b>|42Y");
});

test("render() rejects a registry entry outside the capability grant", () => {
    assert.throws(
        () =>
            render("<% value |> bolder %>", { value: "hi" }, {
                capabilities: ["pure"],
                custom: {
                    entries: [{ namespace: "filter", name: "bolder", capability: "safe-output" }],
                    filter: { bolder: (value) => `<b>${value}</b>` },
                },
            }),
        (error) => error instanceof DejavuError && error.code === "dejavu.template.capability-not-allowed",
    );
});

test("render() rejects a registry entry that shadows a built-in", () => {
    assert.throws(
        () =>
            render("<% value |> upper %>", { value: "hi" }, {
                custom: {
                    entries: [{ namespace: "filter", name: "upper" }],
                    filter: { upper: () => "hijacked" },
                },
            }),
        (error) => error instanceof DejavuError && error.code === "dejavu.template.registry-conflict",
    );
});

test("render() reports a throwing host implementation", () => {
    assert.throws(
        () =>
            render("<% value |> shout %>", { value: "hi" }, {
                custom: {
                    entries: [{ namespace: "filter", name: "shout" }],
                    filter: {
                        shout: () => {
                            throw new Error("host exploded");
                        },
                    },
                },
            }),
        (error) => {
            assert.ok(error instanceof DejavuError);
            assert.equal(error.code, "dejavu.template.custom-invocation");
            assert.match(JSON.stringify(error.args), /host exploded/);
            return true;
        },
    );
});

test("renderIr() raises a structured DejavuError for invalid IR", () => {
    assert.throws(
        () => renderIr("not json"),
        (error) => {
            assert.ok(error instanceof DejavuError, "error is a DejavuError");
            assert.equal(error.severity, "error");
            assert.equal(error.code, "dejavu.ir.invalid-json");
            assert.ok(error.diagnostics.length > 0, "diagnostics is not empty");
            assert.equal(error.diagnostics[0].code, "dejavu.ir.invalid-json");
            return true;
        },
    );
});

test("strictUndefined errors keep code, args, and diagnostics", () => {
    assert.throws(
        () => render("<% missing %>", {}, { strictUndefined: true }),
        (error) => {
            assert.ok(error instanceof DejavuError, "error is a DejavuError");
            assert.equal(error.severity, "error");
            assert.equal(error.code, "dejavu.template.undefined-variable");
            assert.equal(error.args.name, "missing");
            assert.ok(error.diagnostics.length > 0, "diagnostics is not empty");
            assert.equal(
                error.diagnostics[0].code,
                "dejavu.template.undefined-variable",
            );
            return true;
        },
    );
});

test("renderIr() keeps structured errors for valid IR with strictUndefined", () => {
    const irJson = JSON.stringify(parse("<% missing %>"));
    assert.throws(
        () => renderIr(irJson, {}, { strictUndefined: true }),
        (error) => {
            assert.ok(error instanceof DejavuError, "error is a DejavuError");
            assert.equal(error.code, "dejavu.template.undefined-variable");
            assert.equal(error.args.name, "missing");
            assert.ok(error.diagnostics.length > 0, "diagnostics is not empty");
            return true;
        },
    );
});
