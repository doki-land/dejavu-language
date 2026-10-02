/**
 * HTML 转义工具
 */

/**
 * HTML 转义映射表
 */
const HTML_ESCAPE_MAP: Record<string, string> = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#x27;",
};

/**
 * HTML 转义正则表达式
 */
const HTML_ESCAPE_REGEX = /[&<>"']/g;

/**
 * 对字符串进行 HTML 转义
 * @param str 要转义的字符串
 * @returns 转义后的字符串
 */
export function htmlEscape(str: string): string {
    return str.replace(HTML_ESCAPE_REGEX, (char) => HTML_ESCAPE_MAP[char]);
}
