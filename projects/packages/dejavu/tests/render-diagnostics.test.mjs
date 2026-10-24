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
