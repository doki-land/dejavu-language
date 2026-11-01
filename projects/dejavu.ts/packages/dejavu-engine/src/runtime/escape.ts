/**
 * HTML escape — 转义 Dejavu 输出中的 5 个危险字符。
 *
 * 与 Rust `dejavu-types` 的 `html_escape` 过滤器、Node stub 以及
 * legacy TS engine 行为完全一致。只处理 `& < > " '`。
 */
export function htmlEscape(str: string): string {
    return String(str)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#39;");
}

/**
 * 上下文敏感转义（`escape_attr` / `escape_url` / `escape_js` / `escape_css`）。
 *
 * 合同（`dejavu-runtime-topology.md` §0.2，Rust oracle `dejavu-ir`）：四个
 * filter 都是 `Pure`，输出是普通字符串，插值自动转义仍生效；`safe` 只标记
 * HTML text 安全，跨上下文不可互换。
 */

/** `escape_attr`：属性值危险字符集与 `htmlEscape` 相同（`& < > " '`）。 */
export function escapeAttr(str: string): string {
    return htmlEscape(str);
}

/** percent-encode 保留集：RFC 3986 unreserved + reserved（`%` 不在集内 → `%25`）。 */
const PERCENT_KEEP = new Set(
    Array.from(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~:/?#[]@!$&'()*+,;=",
        (c) => c.codePointAt(0) as number,
    ),
);

/** 与 Rust `percent_encode` 对齐：按 UTF-8 字节输出大写 `%XX`。 */
function percentEncode(str: string): string {
    let out = "";
    for (const byte of new TextEncoder().encode(str)) {
        if (PERCENT_KEEP.has(byte)) {
            out += String.fromCharCode(byte);
        } else {
            out += `%${byte.toString(16).toUpperCase().padStart(2, "0")}`;
        }
    }
    return out;
}

/**
 * `escape_url`：scheme allowlist（`http` / `https` / `mailto`，大小写不敏感）
 * 与相对路径放行并 percent-encode；allowlist 外 scheme 与 protocol-relative
 * （`//host`）返回 null（渲染为空串）。TS runtime 没有 Rust 的 `WarningSink`
 * 通道，拒绝时警告丢弃，与 Rust「无 sink」路径一致。
 */
export function escapeUrl(str: string): string | null {
    const colon = str.indexOf(":");
    if (colon !== -1) {
        const head = str.slice(0, colon);
        if (/^[A-Za-z][A-Za-z0-9+.-]*$/.test(head)) {
            const scheme = head.toLowerCase();
            if (scheme === "http" || scheme === "https" || scheme === "mailto") {
                return percentEncode(str);
            }
            return null;
        }
    }
    if (str.startsWith("//")) return null;
    return percentEncode(str);
}

/**
 * `escape_js`：JS 字符串字面量转义，输出含首尾双引号。
 *
 * 与 Rust `escape_js_value` 对齐：`\` `"` 与 C0 控制字符转义；U+2028 /
 * U+2029 必须转义；`< > & ' /` 保持原样——HTML text 转义归插值管道负责。
 */
export function escapeJs(str: string): string {
    let out = '"';
    for (const c of str) {
        switch (c) {
            case "\\":
                out += "\\\\";
                break;
            case '"':
                out += '\\"';
                break;
            case "\b":
                out += "\\b";
                break;
            case "\t":
                out += "\\t";
                break;
            case "\n":
                out += "\\n";
                break;
            case "\f":
                out += "\\f";
                break;
            case "\r":
                out += "\\r";
                break;
            case "\u2028":
                out += "\\u2028";
                break;
            case "\u2029":
                out += "\\u2029";
                break;
            default: {
                const cp = c.codePointAt(0) as number;
                if (cp < 0x20) {
                    out += `\\u${cp.toString(16).padStart(4, "0")}`;
                } else {
                    out += c;
                }
            }
        }
    }
    return `${out}"`;
}

/**
 * `escape_css`：CSS 标识符 / 值转义。
 *
 * 与 Rust `escape_css_value` 对齐：`[A-Za-z0-9_-]` 与码点 ≥ 0x80 原样；
 * `\` → `\\`；其余 → `\` + 小写 hex（无前导零）+ 一个终结空格。
 */
export function escapeCss(str: string): string {
    let out = "";
    for (const c of str) {
        if (/^[A-Za-z0-9]$/.test(c) || c === "_" || c === "-") {
            out += c;
        } else if (c === "\\") {
            out += "\\\\";
        } else if ((c.codePointAt(0) as number) >= 0x80) {
            out += c;
        } else {
            out += `\\${(c.codePointAt(0) as number).toString(16)} `;
        }
    }
    return out;
}
