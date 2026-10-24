import { htmlEscape } from "./escape.js";
import { isTruthy, valueToString } from "./value.js";

/**
 * Dejavu 1.0 合同过滤器集。
 *
 * 与 Rust `dejavu-types` 过滤器系统（见 `projects/crates/dejavu-types/src/values/filter.rs`）
 * 以及 `dejavu-codegen` Node stub 对齐。
 *
 * 未识别的过滤器 **宽松返回原值**（与 Rust 渲染器一致，不抛错）。
 */
export function applyFilter(
    name: string,
    value: unknown,
    args: unknown[],
): unknown {
    switch (name) {
        case "escape":
        case "e":
            return htmlEscape(valueToString(value));

        case "raw":
        case "safe":
        case "s":
            return valueToString(value);

        case "uppercase":
        case "upper":
            return String(value).toUpperCase();

        case "lowercase":
        case "lower":
            return String(value).toLowerCase();

        case "trim":
            return String(value).trim();

        case "default":
            return isTruthy(value) ? value : args[0] ?? "";

        case "length": {
            if (value === null || value === undefined) return 0;
            if (Array.isArray(value)) return value.length;
            if (typeof value === "string") return value.length;
            const maybe = (value as Record<string, unknown>).length;
            return typeof maybe === "number" ? maybe : String(value).length;
        }

        case "join":
            return Array.isArray(value)
                ? value.join(String(args[0] ?? ","))
                : String(value);

        case "replace":
            return String(value).replace(
                (args[0] as string | RegExp) ?? "",
                String(args[1] ?? ""),
            );

        case "date": {
            let d: Date | null = null;
            if (value instanceof Date) {
                d = value;
            } else if (typeof value === "number") {
                d = new Date(value);
            } else if (typeof value === "string") {
                const parsed = new Date(value);
                if (!Number.isNaN(parsed.getTime())) d = parsed;
            }
            if (!d) return "";
            // 第一期只支持 ISO 8601（args[0] 不使用 —— Rust 过滤器默认也是 ISO）。
            return d.toISOString();
        }

        case "slug":
            return slugify(String(value));

        default:
            return value;
    }
}

/**
 * 与 Rust `slug_filter` 对齐：
 *
 * 1. NFKD Unicode 分解
 * 2. 去掉 combining marks
 * 3. 小写
 * 4. 非字母数字 / `-` → `-`
 * 5. 连续 `-` 合并
 * 6. 去首尾 `-`
 */
function slugify(input: string): string {
    const decomposed = input
        .normalize("NFKD")
        .replace(/[\u0300-\u036f]/g, "")
        .toLowerCase();
    const replaced = decomposed.replace(/[^a-z0-9-]+/g, "-");
    return replaced.replace(/-+/g, "-").replace(/^-+|-+$/g, "");
}
