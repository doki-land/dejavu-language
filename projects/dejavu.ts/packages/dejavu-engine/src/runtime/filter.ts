import { htmlEscape } from "./escape.js";
import { valueToString } from "./value.js";

/**
 * Dejavu 1.0 合同过滤器集。
 *
 * 与 Rust `dejavu-ir` 的 `apply_filter` 逐项对齐，所有字符串强制转换都走
 * `valueToString`（null → ""），不使用 JS 的 `String()`。
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
            return value;

        case "uppercase":
        case "upper":
            return valueToString(value).toUpperCase();

        case "lowercase":
        case "lower":
            return valueToString(value).toLowerCase();

        case "trim":
            return valueToString(value).trim();

        case "default": {
            // 只有 null 与空串算缺失 —— 0 / false / [] 保留原值。
            if (value === null || value === undefined || value === "") {
                return args[0] ?? null;
            }
            return value;
        }

        case "length": {
            if (typeof value === "string") return [...value].length;
            if (Array.isArray(value)) return value.length;
            if (value !== null && typeof value === "object") {
                return Object.keys(value as object).length;
            }
            return 0;
        }

        case "join": {
            const sep = valueToString(args[0] ?? ",");
            return Array.isArray(value)
                ? value.map(valueToString).join(sep)
                : valueToString(value);
        }

        case "replace": {
            const from = valueToString(args[0] ?? "");
            const to = valueToString(args[1] ?? "");
            return valueToString(value).split(from).join(to);
        }

        case "date": {
            const pattern = valueToString(args[0] ?? "%Y-%m-%d");
            return formatDate(valueToString(value), pattern);
        }

        case "slug":
            return slugify(valueToString(value));

        default:
            return value;
    }
}

/**
 * 与 Rust `format_date` 对齐的最小 strftime：`%Y %m %d %H %M %S %%`。
 *
 * 输入：空串 → ""；`now` → 当前时间；纯数字 → epoch 毫秒；其他 → ISO 解析。
 */
function formatDate(input: string, pattern: string): string {
    if (input === "") return "";
    let ms: number;
    if (input === "now") {
        ms = Date.now();
    } else if (/^[0-9]+$/.test(input)) {
        ms = Number(input);
    } else {
        ms = Date.parse(input);
        if (Number.isNaN(ms)) return "";
    }
    if (Number.isNaN(ms)) return "";
    const d = new Date(ms);
    const pad = (n: number) => String(n).padStart(2, "0");
    return pattern
        .replaceAll("%%", "\u0000")
        .replaceAll("%Y", String(d.getUTCFullYear()))
        .replaceAll("%m", pad(d.getUTCMonth() + 1))
        .replaceAll("%d", pad(d.getUTCDate()))
        .replaceAll("%H", pad(d.getUTCHours()))
        .replaceAll("%M", pad(d.getUTCMinutes()))
        .replaceAll("%S", pad(d.getUTCSeconds()))
        .replaceAll("\u0000", "%");
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
