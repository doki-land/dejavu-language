import { htmlEscape } from "./escape.js";
import { isTruthy, valueToString } from "./value.js";

/**
 * Dejavu 1.0 支持的二元运算符。
 * 与 Rust `dejavu-ir::IrExprBinaryOp` 枚举一一对应。
 *
 * 行为：
 * - `+` 特殊：任何一侧为字符串时做字符串拼接，否则数值加法。
 * - `&&` / `||` 返回原值（短路），不是布尔。
 * - `in`：R 是 object / array 时检查 key 存在。
 * - 比较与相等用 JS 原生语义。
 */
export function binary(op: string, l: unknown, r: unknown): unknown {
    switch (op) {
        case "+": {
            if (typeof l === "string" || typeof r === "string") {
                return String(l) + String(r);
            }
            return (l as number) + (r as number);
        }
        case "-":
            return (l as number) - (r as number);
        case "*":
            return (l as number) * (r as number);
        case "/":
            return (l as number) / (r as number);
        case "%":
            return (l as number) % (r as number);

        case "==":
            // Dejavu 1.0 只用 ==/!= —— JS 宽松相等
            // eslint-disable-next-line eqeqeq
            return l == r;
        case "!=":
            // eslint-disable-next-line eqeqeq
            return l != r;
        case "===":
            return l === r;
        case "!==":
            return l !== r;

        case "<":
            return (l as any) < (r as any);
        case "<=":
            return (l as any) <= (r as any);
        case ">":
            return (l as any) > (r as any);
        case ">=":
            return (l as any) >= (r as any);

        case "&&":
            return l && r;
        case "||":
            return l || r;

        case "in":
            if (r === null || r === undefined) return false;
            return String(l) in (r as Record<string, unknown>);

        default:
            return null;
    }
}

/**
 * 一元运算符。
 *
 * `!` 使用 **Dejavu truthiness**（不是 JS falsy）。
 */
export function unary(op: string, a: unknown): unknown {
    switch (op) {
        case "!":
            return !isTruthy(a);
        case "-":
            return -(Number(a) || 0);
        case "+":
            return +(Number(a) || 0);
        default:
            return null;
    }
}

/**
 * 成员访问 —— `obj?.[property] ?? null`。
 * 对不存在的键返回 null（不是 undefined）。
 */
export function exprMember(obj: unknown, property: string): unknown {
    if (obj === null || obj === undefined) return null;
    const target = obj as Record<string, unknown>;
    const v = target[property];
    return v === undefined ? null : v;
}

/**
 * 索引访问 —— 数组用数字索引（|0 截断），对象用 String(index)。
 * 越界 / 不存在返回 null。
 */
export function exprIndex(obj: unknown, index: unknown): unknown {
    if (obj === null || obj === undefined) return null;
    if (Array.isArray(obj)) {
        const i = Number(index) | 0;
        const v = obj[i];
        return v === undefined ? null : v;
    }
    if (typeof obj === "object") {
        const v = (obj as Record<string, unknown>)[String(index)];
        return v === undefined ? null : v;
    }
    return null;
}
