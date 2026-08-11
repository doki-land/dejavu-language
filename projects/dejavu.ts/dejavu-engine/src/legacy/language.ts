/**
 * DejaVu 模板引擎语言配置
 */

import { Filter } from "./filter";

/**
 * 语法模式枚举
 */
export enum SyntaxMode {
    /** 编程模式（纯代码） */
    Programming = "Programming",
    /** 模板模式（带插值的文本） */
    Template = "Template",
}

/**
 * 模板配置接口
 */
export interface TemplateConfig {
    /** 代码开始定界符 */
    codeStart: string;
    /** 代码结束定界符 */
    codeEnd: string;
    /** 注释开始定界符 */
    commentStart: string;
    /** 注释结束定界符 */
    commentEnd: string;
    /** 是否支持过滤器管道语法 `a |> b |> c` */
    supportFilterPipe: boolean;
    /** 是否支持遗留的 `for` 语法（否则只允许 `loop`） */
    legacyFor: boolean;
    /** 是否启用 HTML 自动转义（默认 true） */
    autoEscape: boolean;
    /** 最大递归深度，默认 100 */
    maxRecursionDepth: number;
    /** 最大循环迭代次数，默认 10000 */
    maxLoopIterations: number;
    /** 自定义过滤器 */
    customFilters?: Record<string, Filter>;
}

/**
 * 默认模板配置
 */
export const DEFAULT_TEMPLATE_CONFIG: TemplateConfig = {
    codeStart: "<%",
    codeEnd: "%>",
    commentStart: "<#",
    commentEnd: "#>",
    supportFilterPipe: false,
    legacyFor: false,
    autoEscape: true,
    maxRecursionDepth: 100,
    maxLoopIterations: 10000,
};

/**
 * DejaVu 语言配置类
 */
export class DejavuLanguage {
    /** 语法模式 */
    syntaxMode: SyntaxMode = SyntaxMode.Template;
    /** 模板配置 */
    template: TemplateConfig = { ...DEFAULT_TEMPLATE_CONFIG };

    /**
     * 创建新的 DejaVu 语言配置
     */
    constructor() {
        // Properties are already initialized with default values
    }

    /**
     * 设置语法模式
     * @param mode 语法模式
     * @returns 当前实例
     */
    withMode(mode: SyntaxMode): this {
        this.syntaxMode = mode;
        return this;
    }

    /**
     * 设置模板配置
     * @param template 模板配置
     * @returns 当前实例
     */
    withTemplate(template: Partial<TemplateConfig>): this {
        this.template = { ...this.template, ...template };
        return this;
    }

    /**
     * 设置自定义过滤器
     * @param filters 自定义过滤器映射
     * @returns 当前实例
     */
    withCustomFilters(filters: Record<string, Filter>): this {
        this.template.customFilters = { ...this.template.customFilters, ...filters };
        return this;
    }
}

/**
 * 默认 DejaVu 语言配置
 */
export const language = new DejavuLanguage();
