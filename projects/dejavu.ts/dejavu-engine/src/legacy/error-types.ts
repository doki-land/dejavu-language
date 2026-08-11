/**
 * DejaVu 模板引擎错误类型系统
 *
 * 提供结构化的错误类型定义，支持精确的错误位置、错误代码和修复建议。
 */

/**
 * 源代码位置
 */
export interface SourcePosition {
    /** 行号 (1-based) */
    line: number;
    /** 列号 (1-based) */
    column: number;
}

/**
 * 源代码范围
 */
export interface SourceRange {
    /** 起始位置 */
    start: SourcePosition;
    /** 结束位置 */
    end: SourcePosition;
}

/**
 * 错误严重程度
 */
export type ErrorSeverity = "error" | "warning" | "hint";

/**
 * 错误代码枚举
 */
export enum ErrorCode {
    E001_UNEXPECTED_TOKEN = "E001",
    E002_EXPECTED_TOKEN = "E002",
    E003_UNCLOSED_STRING = "E003",
    E004_UNCLOSED_COMMENT = "E004",
    E005_UNCLOSED_DELIMITER = "E005",
    E006_INVALID_IDENTIFIER = "E006",
    E007_INVALID_NUMBER = "E007",
    E008_INVALID_EXPRESSION = "E008",
    E009_MISSING_END = "E009",
    E010_UNDEFINED_VARIABLE = "E010",
    E011_UNDEFINED_FUNCTION = "E011",
    E012_TEMPLATE_NOT_FOUND = "E012",
    E013_INVALID_ASSIGNMENT = "E013",
    E014_INVALID_OPERATOR = "E014",
    E015_MISSING_OPERATOR = "E015",
    E016_INVALID_TYPE = "E016",
    E017_CIRCULAR_REFERENCE = "E017",
    E018_RECURSION_DEPTH_EXCEEDED = "E018",
    E019_LOOP_ITERATION_EXCEEDED = "E019",
    E100_IO_ERROR = "E100",
    E101_CONFIG_ERROR = "E101",
    E102_FILE_NOT_FOUND = "E102",
}

/**
 * 修复建议
 */
export interface ErrorSuggestion {
    /** 建议描述 */
    message: string;
    /** 建议的替换文本（可选） */
    replacement?: string;
    /** 替换范围（可选） */
    range?: SourceRange;
}

/**
 * 结构化错误信息
 */
export interface StructuredError {
    /** 错误代码 */
    code: ErrorCode;
    /** 错误严重程度 */
    severity: ErrorSeverity;
    /** 错误标题 */
    title: string;
    /** 详细错误消息 */
    message: string;
    /** 源代码位置 */
    range?: SourceRange;
    /** 源文件路径 */
    filePath?: string;
    /** 源代码片段 */
    sourceLine?: string;
    /** 修复建议列表 */
    suggestions: ErrorSuggestion[];
    /** 相关文档链接 */
    documentationUrl?: string;
}

/**
 * 解析错误类
 */
export class ParseError extends Error {
    public readonly structured: StructuredError;

    constructor(error: StructuredError) {
        super(error.message);
        this.name = "ParseError";
        this.structured = error;
    }

    /**
     * 格式化为用户友好的字符串
     */
    toString(): string {
        return formatError(this.structured);
    }
}

/**
 * 运行时错误类
 */
export class RuntimeError extends Error {
    public readonly structured: StructuredError;

    constructor(error: StructuredError) {
        super(error.message);
        this.name = "RuntimeError";
        this.structured = error;
    }

    toString(): string {
        return formatError(this.structured);
    }
}

/**
 * 配置错误类
 */
export class ConfigError extends Error {
    public readonly structured: StructuredError;

    constructor(error: StructuredError) {
        super(error.message);
        this.name = "ConfigError";
        this.structured = error;
    }

    toString(): string {
        return formatError(this.structured);
    }
}

/**
 * 错误消息模板
 */
export const ErrorMessages: Record<
    ErrorCode,
    { title: string; template: string; defaultSuggestions: ErrorSuggestion[] }
> = {
    [ErrorCode.E001_UNEXPECTED_TOKEN]: {
        title: "意外的标记",
        template: "意外的标记 {token}，期望 {expected}",
        defaultSuggestions: [
            { message: "检查此处是否有语法错误" },
            { message: "确认是否遗漏了必要的符号" },
        ],
    },
    [ErrorCode.E002_EXPECTED_TOKEN]: {
        title: "缺少标记",
        template: "期望 {expected}，但发现 {found}",
        defaultSuggestions: [
            { message: "添加缺少的标记 {expected}" },
            { message: "检查语法是否正确" },
        ],
    },
    [ErrorCode.E003_UNCLOSED_STRING]: {
        title: "未闭合的字符串",
        template: "字符串字面量未闭合，缺少结束引号",
        defaultSuggestions: [
            { message: '添加结束引号 " 来闭合字符串' },
            { message: "检查字符串中是否有未转义的引号" },
            { message: '如需在字符串中使用引号，请使用 \\" 进行转义' },
        ],
    },
    [ErrorCode.E004_UNCLOSED_COMMENT]: {
        title: "未闭合的注释",
        template: "注释未闭合，缺少结束标记 {end}",
        defaultSuggestions: [
            { message: "添加结束标记 {end} 来闭合注释" },
            { message: "检查注释的开始和结束标记是否匹配" },
        ],
    },
    [ErrorCode.E005_UNCLOSED_DELIMITER]: {
        title: "未闭合的定界符",
        template: "模板定界符未闭合，缺少结束标记 {end}",
        defaultSuggestions: [
            { message: "添加结束标记 {end} 来闭合模板代码块" },
            { message: "检查模板语法是否正确" },
        ],
    },
    [ErrorCode.E006_INVALID_IDENTIFIER]: {
        title: "无效的标识符",
        template: "无效的标识符 {identifier}",
        defaultSuggestions: [
            { message: "标识符必须以字母或下划线开头" },
            { message: "标识符只能包含字母、数字和下划线" },
            { message: "检查是否误用了关键字作为标识符" },
        ],
    },
    [ErrorCode.E007_INVALID_NUMBER]: {
        title: "无效的数字",
        template: "无效的数字字面量 {value}",
        defaultSuggestions: [
            { message: "检查数字格式是否正确" },
            { message: "确保小数点后有数字" },
        ],
    },
    [ErrorCode.E008_INVALID_EXPRESSION]: {
        title: "无效的表达式",
        template: "无效的表达式: {detail}",
        defaultSuggestions: [
            { message: "检查表达式语法是否正确" },
            { message: "确认运算符使用是否正确" },
            { message: "检查括号是否匹配" },
        ],
    },
    [ErrorCode.E009_MISSING_END]: {
        title: "缺少结束语句",
        template: "{construct} 语句缺少对应的 end {construct}",
        defaultSuggestions: [
            { message: "添加 end {construct} 来闭合语句块" },
            { message: "检查语句块的嵌套是否正确" },
        ],
    },
    [ErrorCode.E010_UNDEFINED_VARIABLE]: {
        title: "未定义的变量",
        template: "变量 {name} 未定义",
        defaultSuggestions: [
            { message: "检查变量名是否拼写正确" },
            { message: "确保变量在使用前已声明" },
            { message: "使用 let 关键字声明变量: let {name} = value" },
        ],
    },
    [ErrorCode.E011_UNDEFINED_FUNCTION]: {
        title: "未定义的函数",
        template: "函数 {name} 未定义",
        defaultSuggestions: [
            { message: "检查函数名是否拼写正确" },
            { message: "确保函数已定义" },
            { message: "使用 micro 关键字定义函数" },
        ],
    },
    [ErrorCode.E012_TEMPLATE_NOT_FOUND]: {
        title: "模板未找到",
        template: "模板 {name} 未找到",
        defaultSuggestions: [
            { message: "检查模板名称是否正确" },
            { message: "确保模板文件存在" },
            { message: "检查模板目录配置是否正确" },
        ],
    },
    [ErrorCode.E013_INVALID_ASSIGNMENT]: {
        title: "无效的赋值",
        template: "无法对表达式进行赋值",
        defaultSuggestions: [
            { message: "只能对变量进行赋值" },
            { message: "检查左侧是否为有效的变量名" },
        ],
    },
    [ErrorCode.E014_INVALID_OPERATOR]: {
        title: "无效的运算符",
        template: "运算符 {operator} 在此上下文中无效",
        defaultSuggestions: [
            { message: "检查运算符使用是否正确" },
            { message: "确认操作数类型是否匹配" },
        ],
    },
    [ErrorCode.E015_MISSING_OPERATOR]: {
        title: "缺少运算符",
        template: "表达式缺少运算符",
        defaultSuggestions: [
            { message: "在两个表达式之间添加运算符" },
            { message: "检查是否遗漏了逗号分隔符" },
        ],
    },
    [ErrorCode.E016_INVALID_TYPE]: {
        title: "类型错误",
        template: "类型不匹配: {detail}",
        defaultSuggestions: [
            { message: "检查值的类型是否正确" },
            { message: "使用类型转换函数进行转换" },
        ],
    },
    [ErrorCode.E017_CIRCULAR_REFERENCE]: {
        title: "循环引用",
        template: "检测到循环引用: {chain}",
        defaultSuggestions: [
            { message: "移除循环依赖" },
            { message: "检查模板继承链是否存在循环" },
        ],
    },
    [ErrorCode.E018_RECURSION_DEPTH_EXCEEDED]: {
        title: "递归深度超限",
        template: "递归深度 {depth} 超过了最大限制 {maxDepth}",
        defaultSuggestions: [
            { message: "检查是否存在无限递归" },
            { message: "考虑使用迭代替代递归" },
            { message: "如确需更深递归，可调整 maxRecursionDepth 配置" },
        ],
    },
    [ErrorCode.E019_LOOP_ITERATION_EXCEEDED]: {
        title: "循环迭代次数超限",
        template: "循环迭代次数 {iterations} 超过了最大限制 {maxIterations}",
        defaultSuggestions: [
            { message: "检查是否存在无限循环" },
            { message: "确保循环条件能够正确终止" },
            { message: "如确需更多迭代，可调整 maxLoopIterations 配置" },
        ],
    },
    [ErrorCode.E100_IO_ERROR]: {
        title: "IO 错误",
        template: "文件操作失败: {detail}",
        defaultSuggestions: [{ message: "检查文件权限" }, { message: "确保磁盘空间充足" }],
    },
    [ErrorCode.E101_CONFIG_ERROR]: {
        title: "配置错误",
        template: "配置文件错误: {detail}",
        defaultSuggestions: [
            { message: "检查配置文件格式是否正确" },
            { message: "确保所有必需的配置项都已设置" },
        ],
    },
    [ErrorCode.E102_FILE_NOT_FOUND]: {
        title: "文件未找到",
        template: "文件 {path} 不存在",
        defaultSuggestions: [{ message: "检查文件路径是否正确" }, { message: "确保文件存在" }],
    },
};

/**
 * 创建结构化错误
 */
export function createError(
    code: ErrorCode,
    params: Record<string, string> = {},
    options: {
        range?: SourceRange;
        filePath?: string;
        sourceLine?: string;
        suggestions?: ErrorSuggestion[];
        severity?: ErrorSeverity;
        documentationUrl?: string;
    } = {},
): StructuredError {
    const template = ErrorMessages[code];
    let message = template.template;

    for (const [key, value] of Object.entries(params)) {
        message = message.replace(`{${key}}`, value);
    }

    let suggestions = options.suggestions || [];
    if (suggestions.length === 0 && template.defaultSuggestions) {
        suggestions = template.defaultSuggestions.map((s) => ({
            message: s.message.replace(/\{(\w+)\}/g, (_, key) => params[key] || `{${key}}`),
        }));
    }

    return {
        code,
        severity: options.severity || "error",
        title: template.title,
        message,
        range: options.range,
        filePath: options.filePath,
        sourceLine: options.sourceLine,
        suggestions,
        documentationUrl: options.documentationUrl,
    };
}

/**
 * 格式化错误为用户友好的字符串
 */
export function formatError(error: StructuredError): string {
    const lines: string[] = [];

    const severityIcon = {
        error: "✗",
        warning: "⚠",
        hint: "ℹ",
    };

    const severityLabel = {
        error: "错误",
        warning: "警告",
        hint: "提示",
    };

    lines.push(
        `${severityIcon[error.severity]} ${severityLabel[error.severity]} [${error.code}]: ${error.title}`,
    );
    lines.push(`  ${error.message}`);

    if (error.filePath) {
        let location = error.filePath;
        if (error.range) {
            location += `:${error.range.start.line}:${error.range.start.column}`;
        }
        lines.push(`  位置: ${location}`);
    }

    if (error.sourceLine && error.range) {
        lines.push("");
        lines.push(`  ${error.range.start.line.toString().padStart(4)} | ${error.sourceLine}`);
        const pointer =
            " ".repeat(error.range.start.column - 1) +
            "^".repeat(Math.max(1, error.range.end.column - error.range.start.column));
        lines.push(`      | ${pointer}`);
    }

    if (error.suggestions.length > 0) {
        lines.push("");
        lines.push("  建议:");
        for (const suggestion of error.suggestions) {
            lines.push(`    • ${suggestion.message}`);
            if (suggestion.replacement) {
                lines.push(`      替换为: ${suggestion.replacement}`);
            }
        }
    }

    if (error.documentationUrl) {
        lines.push("");
        lines.push(`  文档: ${error.documentationUrl}`);
    }

    return lines.join("\n");
}

/**
 * 创建意外标记错误
 */
export function unexpectedTokenError(
    found: string,
    expected: string,
    range?: SourceRange,
    sourceLine?: string,
): StructuredError {
    return createError(
        ErrorCode.E001_UNEXPECTED_TOKEN,
        { token: found, expected },
        {
            range,
            sourceLine,
            suggestions: [{ message: `检查此处是否有语法错误，期望 ${expected}` }],
        },
    );
}

/**
 * 创建缺少结束语句错误
 */
export function missingEndError(
    construct: string,
    range?: SourceRange,
    sourceLine?: string,
): StructuredError {
    return createError(
        ErrorCode.E009_MISSING_END,
        { construct },
        {
            range,
            sourceLine,
            suggestions: [{ message: `添加 end ${construct} 来闭合此语句` }],
        },
    );
}

/**
 * 创建未定义变量错误
 */
export function undefinedVariableError(
    name: string,
    range?: SourceRange,
    sourceLine?: string,
): StructuredError {
    return createError(
        ErrorCode.E010_UNDEFINED_VARIABLE,
        { name },
        {
            range,
            sourceLine,
            suggestions: [
                { message: `检查变量名是否拼写正确` },
                { message: `确保变量在使用前已定义` },
            ],
        },
    );
}

/**
 * 创建模板未找到错误
 */
export function templateNotFoundError(name: string): StructuredError {
    return createError(
        ErrorCode.E012_TEMPLATE_NOT_FOUND,
        { name },
        {
            suggestions: [{ message: `检查模板名称是否正确` }, { message: `确保模板已注册` }],
        },
    );
}

/**
 * 创建文件未找到错误
 */
export function fileNotFoundError(path: string): StructuredError {
    return createError(
        ErrorCode.E102_FILE_NOT_FOUND,
        { path },
        {
            suggestions: [{ message: `检查文件路径是否正确` }, { message: `确保文件存在` }],
        },
    );
}

/**
 * 创建配置错误
 */
export function configError(detail: string, suggestions?: ErrorSuggestion[]): StructuredError {
    return createError(
        ErrorCode.E101_CONFIG_ERROR,
        { detail },
        {
            suggestions: suggestions || [{ message: "检查配置文件格式是否正确" }],
        },
    );
}

/**
 * 递归深度超限错误类
 */
export class RecursionDepthError extends Error {
    /** 结构化错误信息 */
    public readonly structured: StructuredError;
    /** 当前递归深度 */
    public readonly depth: number;
    /** 最大递归深度 */
    public readonly maxDepth: number;

    /**
     * 创建递归深度超限错误
     * @param depth 当前递归深度
     * @param maxDepth 最大递归深度
     */
    constructor(depth: number, maxDepth: number) {
        const structured = createError(ErrorCode.E018_RECURSION_DEPTH_EXCEEDED, {
            depth: String(depth),
            maxDepth: String(maxDepth),
        });
        super(structured.message);
        this.name = "RecursionDepthError";
        this.structured = structured;
        this.depth = depth;
        this.maxDepth = maxDepth;
    }

    /**
     * 格式化为用户友好的字符串
     */
    toString(): string {
        return formatError(this.structured);
    }
}

/**
 * 循环迭代次数超限错误类
 */
export class LoopIterationError extends Error {
    /** 结构化错误信息 */
    public readonly structured: StructuredError;
    /** 当前迭代次数 */
    public readonly iterations: number;
    /** 最大迭代次数 */
    public readonly maxIterations: number;

    /**
     * 创建循环迭代次数超限错误
     * @param iterations 当前迭代次数
     * @param maxIterations 最大迭代次数
     */
    constructor(iterations: number, maxIterations: number) {
        const structured = createError(ErrorCode.E019_LOOP_ITERATION_EXCEEDED, {
            iterations: String(iterations),
            maxIterations: String(maxIterations),
        });
        super(structured.message);
        this.name = "LoopIterationError";
        this.structured = structured;
        this.iterations = iterations;
        this.maxIterations = maxIterations;
    }

    /**
     * 格式化为用户友好的字符串
     */
    toString(): string {
        return formatError(this.structured);
    }
}
