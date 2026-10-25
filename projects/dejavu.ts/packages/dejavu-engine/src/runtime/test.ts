/**
 * Dejavu `test` 命名空间：语言中立的谓词表 + 纯函数实现。
 *
 * 与 Rust `dejavu_ir::registry::TESTS` 和 `builtins::call_test` 对齐。
 * 语法为 `expr is name` / `expr is not name` / `expr is name(args)`，
 * 本文件只负责对已求值的左值执行谓词。
 */

import type { CustomRegistry } from "./custom.js";
import { invokeCustom } from "./custom.js";
import { runtimeError } from "./errors.js";
import { isTruthy } from "./value.js";

export interface TestSpec {
    name: string;
    aliases: string[];
    minArgs: number;
    maxArgs: number;
    capability: string;
}

/** 与 Rust `dejavu_ir::registry::TESTS` 同序同构的静态描述。 */
export const TESTS: readonly TestSpec[] = [
    { name: "null", aliases: ["none"], minArgs: 0, maxArgs: 0, capability: "pure" },
    { name: "truthy", aliases: [], minArgs: 0, maxArgs: 0, capability: "pure" },
    { name: "empty", aliases: [], minArgs: 0, maxArgs: 0, capability: "pure" },
    { name: "array", aliases: [], minArgs: 0, maxArgs: 0, capability: "pure" },
    { name: "object", aliases: [], minArgs: 0, maxArgs: 0, capability: "pure" },
    { name: "string", aliases: [], minArgs: 0, maxArgs: 0, capability: "pure" },
    { name: "number", aliases: [], minArgs: 0, maxArgs: 0, capability: "pure" },
    { name: "boolean", aliases: [], minArgs: 0, maxArgs: 0, capability: "pure" },
    { name: "even", aliases: [], minArgs: 0, maxArgs: 0, capability: "pure" },
    { name: "odd", aliases: [], minArgs: 0, maxArgs: 0, capability: "pure" },
];

export function testSpec(name: string): TestSpec | undefined {
    return TESTS.find(
        (spec) => spec.name === name || spec.aliases.includes(name),
    );
}

/** 对已求值的左值执行谓词；`negated` 为 `is not` 形式。 */
export function applyTest(
    name: string,
    value: unknown,
    args: unknown[],
    negated = false,
    custom?: CustomRegistry,
): boolean {
    const spec = testSpec(name);
    if (!spec) {
        // Built-in names win; the compiler already resolved aliases to the
        // canonical name a host-supplied registry keys on.
        const implementation = custom?.test?.[name];
        if (implementation) {
            return invokeCustom("test", name, () => implementation(value, args)) !== negated;
        }
        throw runtimeError("dejavu.template.unknown-test", name, { test: name });
    }
    if (args.length < spec.minArgs || args.length > spec.maxArgs) {
        throw runtimeError(
            "dejavu.template.test-arity",
            `${spec.name} expects ` +
                `${spec.minArgs}..${spec.maxArgs} argument(s), got ${args.length}`,
            {
                test: spec.name,
                minArgs: spec.minArgs,
                maxArgs: spec.maxArgs,
                got: args.length,
            },
        );
    }
    let result: boolean;
    switch (spec.name) {
        case "null":
            result = value === null || value === undefined;
            break;
        case "truthy":
            result = isTruthy(value);
            break;
        case "empty":
            result =
                value === ""
                || (Array.isArray(value) && value.length === 0)
                || (isPlainObject(value) && Object.keys(value).length === 0);
            break;
        case "array":
            result = Array.isArray(value);
            break;
        case "object":
            result = isPlainObject(value);
            break;
        case "string":
            result = typeof value === "string";
            break;
        case "number":
            result = typeof value === "number";
            break;
        case "boolean":
            result = typeof value === "boolean";
            break;
        case "even":
            result = isInteger(value) && value % 2 === 0;
            break;
        case "odd":
            result = isInteger(value) && value % 2 !== 0;
            break;
        default:
            // Unreachable: `testSpec` already rejected unknown names.
            throw runtimeError("dejavu.template.unknown-test", name);
    }
    return result !== negated;
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
    return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isInteger(value: unknown): value is number {
    return typeof value === "number" && Number.isInteger(value);
}
