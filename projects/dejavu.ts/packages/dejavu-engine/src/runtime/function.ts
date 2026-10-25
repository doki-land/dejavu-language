/**
 * Dejavu `function` 命名空间：语言中立的静态表 + 纯函数实现。
 *
 * 与 Rust `dejavu_ir::registry::FUNCTIONS` 和 `builtins::call_function` 对齐。
 * 仅允许 `name(args)` 形式的裸标识符调用；成员调用与未知名字一律报错。
 */

import type { CustomRegistry } from "./custom.js";
import { invokeCustom } from "./custom.js";
import { runtimeError } from "./errors.js";

export interface FunctionSpec {
    name: string;
    aliases: string[];
    minArgs: number;
    maxArgs: number;
    capability: string;
}

/** 与 Rust `dejavu_ir::registry::FUNCTIONS` 同序同构的静态描述。 */
export const FUNCTIONS: readonly FunctionSpec[] = [
    { name: "range", aliases: [], minArgs: 2, maxArgs: 3, capability: "pure" },
    { name: "min", aliases: [], minArgs: 2, maxArgs: 2, capability: "pure" },
    { name: "max", aliases: [], minArgs: 2, maxArgs: 2, capability: "pure" },
];

/** `range` 允许产出的最大元素数，与 Rust `RANGE_LIMIT` 一致。 */
const RANGE_LIMIT = 10_000;

export function functionSpec(name: string): FunctionSpec | undefined {
    return FUNCTIONS.find(
        (spec) => spec.name === name || spec.aliases.includes(name),
    );
}

export function callFunction(
    name: string,
    args: unknown[],
    custom?: CustomRegistry,
): unknown {
    const spec = functionSpec(name);
    if (!spec) {
        // Built-in names win; the canonical spelling emitted by the compiler is
        // the only key a host-supplied registry needs to provide.
        const implementation = custom?.function?.[name];
        if (implementation) return invokeCustom("function", name, () => implementation(args));
        throw runtimeError("dejavu.template.unknown-function", name, { function: name });
    }
    if (args.length < spec.minArgs || args.length > spec.maxArgs) {
        throw runtimeError(
            "dejavu.template.function-arity",
            `${spec.name} expects ` +
                `${spec.minArgs}..${spec.maxArgs} argument(s), got ${args.length}`,
            {
                function: spec.name,
                minArgs: spec.minArgs,
                maxArgs: spec.maxArgs,
                got: args.length,
            },
        );
    }
    switch (spec.name) {
        case "range":
            return range(args);
        case "min":
            return numericPair(args, true);
        case "max":
            return numericPair(args, false);
        default:
            // Unreachable: `functionSpec` already rejected unknown names.
            throw runtimeError("dejavu.template.unknown-function", name);
    }
}

/** `range(start, stop, step=1)`，非法输入按值模型规则返回 null。 */
function range(args: unknown[]): unknown {
    const start = integral(args[0]);
    if (start === undefined) return null;
    const stop = integral(args[1]);
    if (stop === undefined) return null;
    let step = 1;
    if (args.length > 2) {
        const parsed = integral(args[2]);
        if (parsed === undefined || parsed === 0) return null;
        step = parsed;
    }
    const out: number[] = [];
    let current = start;
    while (
        (step > 0 && current < stop) ||
        (step < 0 && current > stop)
    ) {
        if (out.length >= RANGE_LIMIT) return null;
        out.push(current);
        current += step;
    }
    return out;
}

/** `min(a, b)` / `max(a, b)`，仅接受有限数字。 */
function numericPair(args: unknown[], min: boolean): unknown {
    const a = finiteNumber(args[0]);
    const b = finiteNumber(args[1]);
    if (a === undefined || b === undefined) return null;
    return min ? Math.min(a, b) : Math.max(a, b);
}

function finiteNumber(value: unknown): number | undefined {
    if (typeof value !== "number" || !Number.isFinite(value)) return undefined;
    return value;
}

/** 整数（与 Rust `integral` 对齐：非整数、非数字、越界都视为非法）。 */
function integral(value: unknown): number | undefined {
    if (typeof value !== "number" || !Number.isInteger(value)) return undefined;
    if (value < -(2 ** 63) || value > 2 ** 63 - 1) return undefined;
    return value;
}
