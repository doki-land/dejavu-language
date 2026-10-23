/**
 * 1.0 合同的 truthiness —— 与 JS falsy 不同：
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
 * 将任意 JS 值渲染为字符串。与 Rust 渲染器的 debug-print 语义对齐：
 *
 * - null / undefined → ""
 * - boolean → "true" / "false"
 * - number → String()（NaN / Infinity 保持原样）
 * - Date → toISOString()
 * - Array → 递归 valueToString 后 "," join
 * - object → JSON.stringify（fallback 到 String()）
 * - string / 其他 → String()
 */
export function valueToString(v: unknown): string {
    if (v === null || v === undefined) return "";
    if (typeof v === "boolean") return v ? "true" : "false";
    if (typeof v === "number") {
        return String(v);
    }
    if (v instanceof Date) return v.toISOString();
    if (Array.isArray(v)) return v.map(valueToString).join(",");
    if (typeof v === "object") {
        try {
            return JSON.stringify(v);
        } catch {
            return String(v);
        }
    }
    return String(v);
}
