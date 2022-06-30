/**
 * DejaVu 模板引擎词法单元定义
 */

/**
 * 词法单元类型枚举
 */
export enum TokenType {
    // 定界符
    /** 插值开始定界符 */
    DELIMITER_START = "DELIMITER_START",
    /** 插值结束定界符 */
    DELIMITER_END = "DELIMITER_END",
    /** 注释开始定界符 */
    COMMENT_START = "COMMENT_START",
    /** 注释结束定界符 */
    COMMENT_END = "COMMENT_END",

    // 关键字
    /** 变量声明关键字 */
    LET = "LET",
    /** 微函数关键字 */
    MICRO = "MICRO",
    /** 命名空间关键字 */
    NAMESPACE = "NAMESPACE",
    /** 导入关键字 */
    USING = "USING",
    /** 条件语句关键字 */
    IF = "IF",
    /** 条件语句关键字 */
    ELSE = "ELSE",
    /** 循环语句关键字 */
    LOOP = "LOOP",
    /** 遍历关键字 */
    IN = "IN",
    /** 跳出循环关键字 */
    BREAK = "BREAK",
    /** 继续循环关键字 */
    CONTINUE = "CONTINUE",
    /** 匹配语句关键字 */
    MATCH = "MATCH",
    /** 匹配分支关键字 */
    CASE = "CASE",
    /** 结束语句关键字 */
    END = "END",
    /** 类型转换关键字 */
    AS = "AS",
    /** 模板继承关键字 */
    EXTENDS = "EXTENDS",
    /** 块定义关键字 */
    BLOCK = "BLOCK",
    /** 块前追加关键字 */
    PREPEND = "PREPEND",
    /** 块后追加关键字 */
    APPEND = "APPEND",
    /** 父块调用关键字 */
    SUPER = "SUPER",
    /** Include 关键字 */
    INCLUDE = "INCLUDE",
    /** With 关键字 */
    WITH = "WITH",

    // 标识符
    /** 标识符 */
    IDENTIFIER = "IDENTIFIER",

    // 字面量
    /** 布尔字面量 */
    BOOLEAN_LITERAL = "BOOLEAN_LITERAL",
    /** 数字字面量 */
    NUMBER_LITERAL = "NUMBER_LITERAL",
    /** 字符字面量 */
    CHARACTER_LITERAL = "CHARACTER_LITERAL",
    /** 字符串字面量 */
    STRING_LITERAL = "STRING_LITERAL",
    /** 原始标识符字面量 */
    RAW_IDENTIFIER = "RAW_IDENTIFIER",

    // 运算符
    /** 赋值运算符 */
    ASSIGN = "ASSIGN",
    /** 加法运算符 */
    PLUS = "PLUS",
    /** 减法运算符 */
    MINUS = "MINUS",
    /** 乘法运算符 */
    MULTIPLY = "MULTIPLY",
    /** 除法运算符 */
    DIVIDE = "DIVIDE",
    /** 取模运算符 */
    MODULO = "MODULO",
    /** 逻辑与运算符 */
    LOGICAL_AND = "LOGICAL_AND",
    /** 逻辑或运算符 */
    LOGICAL_OR = "LOGICAL_OR",
    /** 逻辑非运算符 */
    LOGICAL_NOT = "LOGICAL_NOT",
    /** 等于运算符 */
    EQUAL = "EQUAL",
    /** 不等于运算符 */
    NOT_EQUAL = "NOT_EQUAL",
    /** 小于运算符 */
    LESS_THAN = "LESS_THAN",
    /** 大于运算符 */
    GREATER_THAN = "GREATER_THAN",
    /** 小于等于运算符 */
    LESS_THAN_OR_EQUAL = "LESS_THAN_OR_EQUAL",
    /** 大于等于运算符 */
    GREATER_THAN_OR_EQUAL = "GREATER_THAN_OR_EQUAL",
    /** 管道运算符 */
    PIPE = "PIPE",

    // 标点符号
    /** 左括号 */
    LEFT_PAREN = "LEFT_PAREN",
    /** 右括号 */
    RIGHT_PAREN = "RIGHT_PAREN",
    /** 左方括号 */
    LEFT_BRACKET = "LEFT_BRACKET",
    /** 右方括号 */
    RIGHT_BRACKET = "RIGHT_BRACKET",
    /** 左花括号 */
    LEFT_BRACE = "LEFT_BRACE",
    /** 右花括号 */
    RIGHT_BRACE = "RIGHT_BRACE",
    /** 分号 */
    SEMICOLON = "SEMICOLON",
    /** 逗号 */
    COMMA = "COMMA",
    /** 点号 */
    DOT = "DOT",
    /** 冒号 */
    COLON = "COLON",
    /** 双冒号 */
    DOUBLE_COLON = "DOUBLE_COLON",
    /** 范围运算符 */
    RANGE = "RANGE",

    // 其他
    /** 文本 */
    TEXT = "TEXT",
    /** 注释 */
    COMMENT = "COMMENT",
    /** 结束标记 */
    EOF = "EOF",
}

/**
 * 词法单元接口
 */
export interface Token {
    /** 词法单元类型 */
    type: TokenType;
    /** 词法单元值 */
    value: string;
    /** 位置信息 */
    loc: {
        /** 开始位置 */
        start: {
            line: number;
            column: number;
        };
        /** 结束位置 */
        end: {
            line: number;
            column: number;
        };
    };
}

/**
 * 创建词法单元
 * @param type 词法单元类型
 * @param value 词法单元值
 * @param startLine 开始行号
 * @param startColumn 开始列号
 * @param endLine 结束行号
 * @param endColumn 结束列号
 * @returns 词法单元
 */
export function createToken(
    type: TokenType,
    value: string,
    startLine: number,
    startColumn: number,
    endLine: number,
    endColumn: number,
): Token {
    return {
        type,
        value,
        loc: {
            start: {
                line: startLine,
                column: startColumn,
            },
            end: {
                line: endLine,
                column: endColumn,
            },
        },
    };
}
