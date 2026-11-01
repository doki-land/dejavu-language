import { htmlEscape } from "./escape.js";
import { runtimeError } from "./errors.js";
import { isTruthy, valueToString, valuesEqual } from "./value.js";

/**
 * Dejavu v0 支持的二元运算符。
 * 与 Rust `dejavu-ir` 的 `eval_binary` 一一对应，**不借用 JS 隐式转换**：
 *
 * - `+`：两侧都是 number 时相加，否则按 `valueToString` 做字符串拼接。
 * - `-` `*` `/` `%`：只接受 number，非 number 或结果非有限数 → null。
 * - `==` / `!=`：结构相等（见 `valuesEqual`），不做类型转换。
 * - `<` `<=` `>` `>=`：两侧都是 number 才比较，否则 false。
 * - `&&` / `||`：返回 **boolean**（Dejavu truthiness），不是原值。
 * - `in`：数组按成员、字符串按子串、对象按 key 判断。
 */
export function binary(op: string, l: unknown, r: unknown): unknown {
    switch (op) {
        case "+": {
            if (typeof l === "number" && typeof r === "number") {
                return finite(l + r);
            }
            return valueToString(l) + valueToString(r);
        }
        case "-":
        case "*":
        case "/":
        case "%": {
            if (typeof l !== "number" || typeof r !== "number") return null;
            const value =
                op === "-"
                    ? l - r
                    : op === "*"
                      ? l * r
                      : op === "/"
                        ? l / r
                        : l % r;
            return finite(value);
        }

        case "==":
            return valuesEqual(l, r);
        case "!=":
            return !valuesEqual(l, r);

        case "<":
        case "<=":
        case ">":
        case ">=": {
            if (typeof l !== "number" || typeof r !== "number") return false;
            if (!Number.isFinite(l) || !Number.isFinite(r)) return false;
            switch (op) {
                case "<":
                    return l < r;
                case "<=":
                    return l <= r;
                case ">":
                    return l > r;
                default:
                    return l >= r;
            }
        }

        case "&&":
            return isTruthy(l) && isTruthy(r);
        case "||":
            return isTruthy(l) || isTruthy(r);

        case "in":
            if (Array.isArray(r)) return r.some((item) => valuesEqual(item, l));
            if (typeof r === "string") return r.includes(valueToString(l));
            if (r !== null && typeof r === "object") {
                return Object.prototype.hasOwnProperty.call(r, valueToString(l));
            }
            return false;

        default:
            return null;
    }
}

function finite(value: number): number | null {
    return Number.isFinite(value) ? value : null;
}

/**
 * 一元运算符 —— 与 Rust `eval_unary` 对齐。
 *
 * `!` 使用 Dejavu truthiness；`-` 只接受 number；`+` 是恒等。
 */
export function unary(op: string, a: unknown): unknown {
    switch (op) {
        case "!":
            return !isTruthy(a);
        case "-":
            return typeof a === "number" ? finite(-a) : null;
        case "+":
            return a;
        default:
            return null;
    }
}

/**
 * 标识符读取 —— 缺失键归一 null（不是 undefined）。
 *
 * `strict` 是 render 期标志（生成代码不固化）：开启时缺失标识符抛
 * `dejavu.template.undefined-variable`，args 携带 `name`。
 */
export function exprIdent(ctx: Record<string, unknown>, name: string, strict: boolean): unknown {
    if (Object.prototype.hasOwnProperty.call(ctx, name)) return ctx[name];
    if (strict) throw runtimeError("dejavu.template.undefined-variable", name, { name });
    return null;
}

/**
 * 成员访问 —— 对不存在的键返回 null（不是 undefined）。
 *
 * strict 只对「缺失成员」报错：接收者是非 null 非数组对象且没有该 own 键时
 * 抛 `dejavu.template.undefined-member`，args 携带 `name` 与语法级 `path`。
 * null / 非对象接收者仍按缺失归一 null 宽容处理。
 */
export function exprMember(obj: unknown, property: string, strict: boolean, path: string): unknown {
    if (obj === null || obj === undefined) return null;
    if (strict && typeof obj === "object" && !Array.isArray(obj)) {
        if (!Object.prototype.hasOwnProperty.call(obj, property)) {
            throw runtimeError("dejavu.template.undefined-member", property, { name: property, path });
        }
    }
    const target = obj as Record<string, unknown>;
    const v = target[property];
    return v === undefined ? null : v;
}

/**
 * 索引访问 —— 数组用数字索引（|0 截断），对象用 String(index)。
 * 越界 / 不存在返回 null。
 *
 * strict 只对「真正执行了查找」的组合报错：非负整数越界数组下标抛
 * `dejavu.template.undefined-index`（args 携带 `name` 与语法级 `path`）；
 * 字符串键在对象上缺失同样抛 `undefined-index`。负数 / 小数 / 类型不匹配的
 * 下标保持原有 lenient 归一，即使 strict 开启也不报。
 */
export function exprIndex(obj: unknown, index: unknown, strict: boolean, path: string): unknown {
    if (obj === null || obj === undefined) return null;
    if (Array.isArray(obj)) {
        if (
            strict &&
            typeof index === "number" &&
            Number.isInteger(index) &&
            index >= 0 &&
            index >= obj.length
        ) {
            throw runtimeError("dejavu.template.undefined-index", String(index), {
                name: String(index),
                path,
            });
        }
        const i = Number(index) | 0;
        const v = obj[i];
        return v === undefined ? null : v;
    }
    if (typeof obj === "object") {
        const key = String(index);
        if (strict && typeof index === "string" && !Object.prototype.hasOwnProperty.call(obj, key)) {
            throw runtimeError("dejavu.template.undefined-index", key, { name: key, path });
        }
        const v = (obj as Record<string, unknown>)[key];
        return v === undefined ? null : v;
    }
    return null;
}
