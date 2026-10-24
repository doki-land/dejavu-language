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
