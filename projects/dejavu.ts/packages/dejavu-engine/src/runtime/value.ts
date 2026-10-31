/**
 * v0 合同的 truthiness —— 与 JS falsy 不同：
 *
 * - null / undefined → false
 * - "" → false
 * - 0 (number) / false (boolean) → false
 * - 空数组 [] → false
 * - 空对象 {} → false
 * - "0" (字符串) → **true**
 * - 其他 → true
 */
export function isTruthy(v: unknown): boolean {
    if (v === null || v === undefined) return false;
    if (v === "") return false;
    if (v === false) return false;
    if (v === 0) return false;
    if (Array.isArray(v) && v.length === 0) return false;
    if (typeof v === "object" && !Array.isArray(v)) {
        return Object.keys(v as object).length > 0;
    }
    return true;
}

/**
 * 将任意 JS 值渲染为字符串。与 Rust `value_to_string` 对齐：
 *
 * - null / undefined → ""
 * - boolean → "true" / "false"
 * - number → String()（NaN / Infinity 保持原样）
 * - Date → toISOString()
 * - Array / object → 紧凑 JSON（与 serde_json 输出一致）
 * - string / 其他 → String()
 */
export function valueToString(v: unknown): string {
    if (v === null || v === undefined) return "";
    if (typeof v === "boolean") return v ? "true" : "false";
    if (typeof v === "number") {
        return String(v);
    }
    if (v instanceof Date) return v.toISOString();
    if (Array.isArray(v) || typeof v === "object") {
        try {
            return JSON.stringify(v);
        } catch {
            return String(v);
        }
    }
    return String(v);
}

/**
 * 结构相等 —— 与 Rust `serde_json::Value` 的 `==` 对齐，不做类型转换。
 */
export function valuesEqual(a: unknown, b: unknown): boolean {
    if (a === b) return true;
    if (a === null || b === null || a === undefined || b === undefined) return false;
    if (Array.isArray(a) && Array.isArray(b)) {
        return a.length === b.length && a.every((v, i) => valuesEqual(v, b[i]));
    }
    if (Array.isArray(a) !== Array.isArray(b)) return false;
    if (typeof a === "object" && typeof b === "object") {
        const ka = Object.keys(a as object);
        const kb = Object.keys(b as object);
        if (ka.length !== kb.length) return false;
        if (ka.some((k) => !Object.prototype.hasOwnProperty.call(b, k))) return false;
        return ka.every((k) =>
            valuesEqual(
                (a as Record<string, unknown>)[k],
                (b as Record<string, unknown>)[k],
            ),
        );
    }
    return false;
}
