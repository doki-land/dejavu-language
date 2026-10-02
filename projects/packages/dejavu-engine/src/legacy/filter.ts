/**
 * DejaVu 模板引擎过滤器系统
 */

/**
 * 过滤器函数类型
 * @param value 输入值
 * @param args 过滤器参数
 * @returns 过滤后的值
 */
export type Filter = (value: any, args: any[]) => any;

/**
 * 过滤器注册表类
 * 用于管理和应用模板过滤器
 */
export class FilterRegistry {
    /**
     * 已注册的过滤器映射
     */
    private filters: Map<string, Filter>;

    /**
     * 创建过滤器注册表实例
     */
    constructor() {
        this.filters = new Map<string, Filter>();
        this.registerBuiltinFilters();
    }

    /**
     * 注册过滤器
     * @param name 过滤器名称
     * @param filter 过滤器函数
     */
    register(name: string, filter: Filter): void {
        this.filters.set(name, filter);
    }

    /**
     * 获取过滤器
     * @param name 过滤器名称
     * @returns 过滤器函数，如果不存在则返回 undefined
     */
    get(name: string): Filter | undefined {
        return this.filters.get(name);
    }

    /**
     * 检查过滤器是否存在
     * @param name 过滤器名称
     * @returns 是否存在
     */
    has(name: string): boolean {
        return this.filters.has(name);
    }

    /**
     * 应用过滤器
     * @param name 过滤器名称
     * @param value 输入值
     * @param args 过滤器参数
     * @returns 过滤后的值
     */
    apply(name: string, value: any, args: any[]): any {
        const filter = this.filters.get(name);
        if (!filter) {
            throw new Error(`过滤器未找到: ${name}`);
        }
        return filter(value, args);
    }

    /**
     * 注册内置过滤器
     */
    private registerBuiltinFilters(): void {
        this.register("uppercase", uppercaseFilter);
        this.register("lowercase", lowercaseFilter);
        this.register("trim", trimFilter);
        this.register("default", defaultFilter);
        this.register("length", lengthFilter);
        this.register("join", joinFilter);
        this.register("split", splitFilter);
        this.register("replace", replaceFilter);
        this.register("first", firstFilter);
        this.register("last", lastFilter);
        this.register("sort", sortFilter);
        this.register("reverse", reverseFilter);
        this.register("abs", absFilter);
        this.register("round", roundFilter);
        this.register("floor", floorFilter);
        this.register("ceil", ceilFilter);
        this.register("capitalize", capitalizeFilter);
        this.register("title", titleFilter);
        this.register("striptags", striptagsFilter);
        this.register("format", formatFilter);
        this.register("bool", boolFilter);
        this.register("not", notFilter);
    }
}

/**
 * 大写过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 大写字符串
 */
function uppercaseFilter(value: any, args: any[]): any {
    if (typeof value === "string") {
        return value.toUpperCase();
    }
    return value;
}

/**
 * 小写过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 小写字符串
 */
function lowercaseFilter(value: any, args: any[]): any {
    if (typeof value === "string") {
        return value.toLowerCase();
    }
    return value;
}

/**
 * 去除首尾空白过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 去除首尾空白后的字符串
 */
function trimFilter(value: any, args: any[]): any {
    if (typeof value === "string") {
        return value.trim();
    }
    return value;
}

/**
 * 默认值过滤器
 * @param value 输入值
 * @param args 参数，第一个参数为默认值
 * @returns 如果值为假则返回默认值
 */
function defaultFilter(value: any, args: any[]): any {
    if (isTruthy(value)) {
        return value;
    }
    return args.length > 0 ? args[0] : "";
}

/**
 * 长度过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 长度
 */
function lengthFilter(value: any, args: any[]): any {
    if (typeof value === "string") {
        return value.length;
    }
    if (Array.isArray(value)) {
        return value.length;
    }
    if (value && typeof value === "object") {
        return Object.keys(value).length;
    }
    return 0;
}

/**
 * 连接过滤器
 * @param value 输入值
 * @param args 参数，第一个参数为分隔符
 * @returns 连接后的字符串
 */
function joinFilter(value: any, args: any[]): any {
    const separator = args.length > 0 ? String(args[0]) : "";
    if (Array.isArray(value)) {
        return value.map((item) => String(item)).join(separator);
    }
    return "";
}

/**
 * 分割过滤器
 * @param value 输入值
 * @param args 参数，第一个参数为分隔符
 * @returns 分割后的数组
 */
function splitFilter(value: any, args: any[]): any {
    if (typeof value === "string") {
        const separator = args.length > 0 ? String(args[0]) : "";
        return value.split(separator);
    }
    return value;
}

/**
 * 替换过滤器
 * @param value 输入值
 * @param args 参数，第一个为搜索字符串，第二个为替换字符串
 * @returns 替换后的字符串
 */
function replaceFilter(value: any, args: any[]): any {
    if (typeof value === "string") {
        if (args.length >= 2) {
            const search = String(args[0]);
            const replacement = String(args[1]);
            return value.split(search).join(replacement);
        }
        return value;
    }
    return value;
}

/**
 * 第一个元素过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 数组的第一个元素
 */
function firstFilter(value: any, args: any[]): any {
    if (Array.isArray(value)) {
        return value.length > 0 ? value[0] : null;
    }
    return value;
}

/**
 * 最后一个元素过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 数组的最后一个元素
 */
function lastFilter(value: any, args: any[]): any {
    if (Array.isArray(value)) {
        return value.length > 0 ? value[value.length - 1] : null;
    }
    return value;
}

/**
 * 排序过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 排序后的数组
 */
function sortFilter(value: any, args: any[]): any {
    if (Array.isArray(value)) {
        return [...value].sort((a, b) => {
            const aStr = String(a);
            const bStr = String(b);
            if (aStr < bStr) return -1;
            if (aStr > bStr) return 1;
            return 0;
        });
    }
    return value;
}

/**
 * 反转过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 反转后的数组或字符串
 */
function reverseFilter(value: any, args: any[]): any {
    if (Array.isArray(value)) {
        return [...value].reverse();
    }
    if (typeof value === "string") {
        return value.split("").reverse().join("");
    }
    return value;
}

/**
 * 绝对值过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 绝对值
 */
function absFilter(value: any, args: any[]): any {
    if (typeof value === "number") {
        return Math.abs(value);
    }
    return value;
}

/**
 * 四舍五入过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 四舍五入后的整数
 */
function roundFilter(value: any, args: any[]): any {
    if (typeof value === "number") {
        return Math.round(value);
    }
    return value;
}

/**
 * 向下取整过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 向下取整后的整数
 */
function floorFilter(value: any, args: any[]): any {
    if (typeof value === "number") {
        return Math.floor(value);
    }
    return value;
}

/**
 * 向上取整过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 向上取整后的整数
 */
function ceilFilter(value: any, args: any[]): any {
    if (typeof value === "number") {
        return Math.ceil(value);
    }
    return value;
}

/**
 * 首字母大写过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 首字母大写的字符串
 */
function capitalizeFilter(value: any, args: any[]): any {
    if (typeof value === "string" && value.length > 0) {
        return value.charAt(0).toUpperCase() + value.slice(1);
    }
    return value;
}

/**
 * 标题格式过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 每个单词首字母大写的字符串
 */
function titleFilter(value: any, args: any[]): any {
    if (typeof value === "string") {
        return value
            .split(/\s+/)
            .map((word) => {
                if (word.length > 0) {
                    return word.charAt(0).toUpperCase() + word.slice(1);
                }
                return word;
            })
            .join(" ");
    }
    return value;
}

/**
 * 去除 HTML 标签过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 去除 HTML 标签后的字符串
 */
function striptagsFilter(value: any, args: any[]): any {
    if (typeof value === "string") {
        let result = "";
        let inTag = false;
        for (const char of value) {
            if (char === "<") {
                inTag = true;
            } else if (char === ">") {
                inTag = false;
            } else if (!inTag) {
                result += char;
            }
        }
        return result;
    }
    return value;
}

/**
 * 格式化过滤器
 * @param value 输入值（格式字符串）
 * @param args 参数列表
 * @returns 格式化后的字符串
 */
function formatFilter(value: any, args: any[]): any {
    if (typeof value === "string") {
        let result = value;
        for (let i = 0; i < args.length; i++) {
            const placeholder = `{${i}}`;
            result = result.split(placeholder).join(String(args[i]));
        }
        return result;
    }
    return value;
}

/**
 * 布尔值过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 布尔值
 */
function boolFilter(value: any, args: any[]): any {
    return isTruthy(value);
}

/**
 * 逻辑非过滤器
 * @param value 输入值
 * @param args 参数（未使用）
 * @returns 逻辑非结果
 */
function notFilter(value: any, args: any[]): any {
    return !isTruthy(value);
}

/**
 * 判断值是否为真值
 * @param value 输入值
 * @returns 是否为真值
 */
function isTruthy(value: any): boolean {
    if (value === null || value === undefined) {
        return false;
    }
    if (typeof value === "boolean") {
        return value;
    }
    if (typeof value === "number") {
        return value !== 0;
    }
    if (typeof value === "string") {
        return value.length > 0;
    }
    if (Array.isArray(value)) {
        return value.length > 0;
    }
    if (typeof value === "object") {
        return Object.keys(value).length > 0;
    }
    return true;
}

/**
 * 默认过滤器注册表实例
 */
export const filterRegistry = new FilterRegistry();
