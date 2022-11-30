/**
 * DejaVu 模板引擎词法分析器
 */

import {Token, TokenType, createToken} from "./token";
import {DejavuLanguage, language as defaultLanguage} from "./language";

/**
 * 词法分析器类
 */
export class Lexer {
    /** 模板源代码 */
    public readonly source: string;
    /** 当前位置 */
    private position: number;
    /** 当前行号 */
    private line: number;
    /** 当前列号 */
    private column: number;
    /** 定界符配置 */
    private delimiters: {
        interpolate: [string, string];
        comment: [string, string];
    };
    /** 是否在模板表达式内部 */
    private inTemplateExpression: boolean;

    /**
     * 构造函数
     * @param source 模板源代码
     * @param language 语言配置
     */
    constructor(source: string, language?: DejavuLanguage) {
        this.source = source;
        this.position = 0;
        this.line = 1;
        this.column = 1;
        this.inTemplateExpression = false;
        const lang = language || defaultLanguage;
        this.delimiters = {
            interpolate: [lang.template.codeStart, lang.template.codeEnd] as [string, string],
            comment: [lang.template.commentStart, lang.template.commentEnd] as [string, string],
        };
    }

    /**
     * 查看下一个词法单元（不前进）
     * @returns 词法单元
     */
    peekToken(): Token {
        const savedPosition = this.position;
        const savedLine = this.line;
        const savedColumn = this.column;
        const savedInTemplateExpression = this.inTemplateExpression;

        const token = this.nextToken();

        this.position = savedPosition;
        this.line = savedLine;
        this.column = savedColumn;
        this.inTemplateExpression = savedInTemplateExpression;

        return token;
    }

    /**
     * 获取下一个词法单元
     * @returns 词法单元
     */
    nextToken(): Token {
        if (!this.inTemplateExpression) {
            this.skipWhitespace();
        } else {
            this.skipWhitespaceInTemplate();
        }

        if (this.position >= this.source.length) {
            return this.createToken(
                TokenType.EOF,
                "",
                this.line,
                this.column,
                this.line,
                this.column,
            );
        }

        const commentToken = this.checkComment();
        if (commentToken) {
            return commentToken;
        }

        const delimiterToken = this.checkDelimiters();
        if (delimiterToken) {
            return delimiterToken;
        }

        if (!this.inTemplateExpression) {
            const textToken = this.checkText();
            if (textToken) {
                return textToken;
            }
        }

        const identifierToken = this.checkIdentifier();
        if (identifierToken) {
            return identifierToken;
        }

        const literalToken = this.checkLiteral();
        if (literalToken) {
            return literalToken;
        }

        const operatorToken = this.checkOperator();
        if (operatorToken) {
            return operatorToken;
        }

        const char = this.source[this.position];
        const token = this.createToken(
            TokenType.TEXT,
            char,
            this.line,
            this.column,
            this.line,
            this.column + 1,
        );
        this.position++;
        this.column++;
        return token;
    }

    /**
     * 跳过空白字符
     */
    private skipWhitespace(): void {
        while (this.position < this.source.length) {
            const char = this.source[this.position];
            if (char === " " || char === "\t" || char === "\r") {
                this.position++;
                this.column++;
            } else if (char === "\n") {
                this.position++;
                this.line++;
                this.column = 1;
            } else {
                break;
            }
        }
    }

    /**
     * 跳过模板表达式内的空白字符（保留换行符信息）
     */
    private skipWhitespaceInTemplate(): void {
        while (this.position < this.source.length) {
            const char = this.source[this.position];
            if (char === " " || char === "\t" || char === "\r") {
                this.position++;
                this.column++;
            } else if (char === "\n") {
                this.position++;
                this.line++;
                this.column = 1;
            } else {
                break;
            }
        }
    }

    /**
     * 检查定界符
     * @returns 词法单元，如果不是定界符则返回null
     */
    private checkDelimiters(): Token | null {
        if (this.source.startsWith(this.delimiters.interpolate[0], this.position)) {
            const startLine = this.line;
            const startColumn = this.column;
            const length = this.delimiters.interpolate[0].length;
            this.position += length;
            this.column += length;
            this.inTemplateExpression = true;
            return this.createToken(
                TokenType.DELIMITER_START,
                this.delimiters.interpolate[0],
                startLine,
                startColumn,
                this.line,
                this.column,
            );
        }

        if (this.source.startsWith(this.delimiters.interpolate[1], this.position)) {
            const startLine = this.line;
            const startColumn = this.column;
            const length = this.delimiters.interpolate[1].length;
            this.position += length;
            this.column += length;
            this.inTemplateExpression = false;
            return this.createToken(
                TokenType.DELIMITER_END,
                this.delimiters.interpolate[1],
                startLine,
                startColumn,
                this.line,
                this.column,
            );
        }

        // 检查注释开始定界符
        if (this.source.startsWith(this.delimiters.comment[0], this.position)) {
            const startLine = this.line;
            const startColumn = this.column;
            const length = this.delimiters.comment[0].length;
            this.position += length;
            this.column += length;
            return this.createToken(
                TokenType.COMMENT_START,
                this.delimiters.comment[0],
                startLine,
                startColumn,
                this.line,
                this.column,
            );
        }

        // 检查注释结束定界符
        if (this.source.startsWith(this.delimiters.comment[1], this.position)) {
            const startLine = this.line;
            const startColumn = this.column;
            const length = this.delimiters.comment[1].length;
            this.position += length;
            this.column += length;
            return this.createToken(
                TokenType.COMMENT_END,
                this.delimiters.comment[1],
                startLine,
                startColumn,
                this.line,
                this.column,
            );
        }

        return null;
    }

    /**
     * 检查注释
     * @returns 词法单元，如果不是注释则返回null
     */
    private checkComment(): Token | null {
        // 检查注释开始定界符
        if (!this.source.startsWith(this.delimiters.comment[0], this.position)) {
            return null;
        }

        // 找到注释结束定界符
        const startPos = this.position;
        const startLine = this.line;
        const startColumn = this.column;
        const endPos = this.source.indexOf(
            this.delimiters.comment[1],
            startPos + this.delimiters.comment[0].length,
        );

        if (endPos === -1) {
            // 注释未闭合，视为文本
            return null;
        }

        // 计算注释内容
        const commentContent = this.source.substring(
            startPos + this.delimiters.comment[0].length,
            endPos,
        );
        const endLine = this.line + (commentContent.match(/\n/g) || []).length;
        const lastNewlineIndex = commentContent.lastIndexOf("\n");
        const endColumn =
            lastNewlineIndex === -1
                ? startColumn + commentContent.length + this.delimiters.comment[0].length
                : commentContent.length - lastNewlineIndex;

        // 更新位置
        this.position = endPos + this.delimiters.comment[1].length;
        this.line = endLine;
        this.column = endColumn + this.delimiters.comment[1].length;

        return this.createToken(
            TokenType.COMMENT,
            commentContent,
            startLine,
            startColumn,
            endLine,
            endColumn,
        );
    }

    /**
     * 检查标识符和关键字
     * @returns 词法单元，如果不是标识符或关键字则返回null
     */
    private checkIdentifier(): Token | null {
        const startPos = this.position;
        const startLine = this.line;
        const startColumn = this.column;

        // 检查标识符的第一个字符
        if (!this.isIdentifierStart(this.source[this.position])) {
            return null;
        }

        // 读取标识符
        this.position++;
        this.column++;
        while (
            this.position < this.source.length &&
            this.isIdentifierPart(this.source[this.position])
            ) {
            this.position++;
            this.column++;
        }

        const identifier = this.source.substring(startPos, this.position);

        // 检查是否为关键字
        const keywordToken = this.checkKeyword(identifier, startLine, startColumn);
        if (keywordToken) {
            return keywordToken;
        }

        // 检查是否为布尔字面量
        if (identifier === "true" || identifier === "false") {
            return this.createToken(
                TokenType.BOOLEAN_LITERAL,
                identifier,
                startLine,
                startColumn,
                this.line,
                this.column,
            );
        }

        // 普通标识符
        return this.createToken(
            TokenType.IDENTIFIER,
            identifier,
            startLine,
            startColumn,
            this.line,
            this.column,
        );
    }

    /**
     * 检查是否为标识符的开始字符
     * @param char 字符
     * @returns 是否为标识符的开始字符
     */
    private isIdentifierStart(char: string): boolean {
        return /^[a-zA-Z_]$/.test(char);
    }

    /**
     * 检查是否为标识符的组成字符
     * @param char 字符
     * @returns 是否为标识符的组成字符
     */
    private isIdentifierPart(char: string): boolean {
        return /^[a-zA-Z0-9_]$/.test(char);
    }

    /**
     * 检查是否为关键字
     * @param identifier 标识符
     * @param startLine 开始行号
     * @param startColumn 开始列号
     * @returns 词法单元，如果不是关键字则返回null
     */
    private checkKeyword(identifier: string, startLine: number, startColumn: number): Token | null {
        switch (identifier) {
            case "let":
                return this.createToken(
                    TokenType.LET,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "micro":
                return this.createToken(
                    TokenType.MICRO,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "namespace":
                return this.createToken(
                    TokenType.NAMESPACE,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "using":
                return this.createToken(
                    TokenType.USING,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "if":
                return this.createToken(
                    TokenType.IF,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "else":
                return this.createToken(
                    TokenType.ELSE,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "loop":
                return this.createToken(
                    TokenType.LOOP,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "in":
                return this.createToken(
                    TokenType.IN,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "break":
                return this.createToken(
                    TokenType.BREAK,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "continue":
                return this.createToken(
                    TokenType.CONTINUE,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "match":
                return this.createToken(
                    TokenType.MATCH,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "case":
                return this.createToken(
                    TokenType.CASE,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "end":
                return this.createToken(
                    TokenType.END,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "as":
                return this.createToken(
                    TokenType.AS,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "extends":
                return this.createToken(
                    TokenType.EXTENDS,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "block":
                return this.createToken(
                    TokenType.BLOCK,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "prepend":
                return this.createToken(
                    TokenType.PREPEND,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "append":
                return this.createToken(
                    TokenType.APPEND,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "super":
                return this.createToken(
                    TokenType.SUPER,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "include":
                return this.createToken(
                    TokenType.INCLUDE,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            case "with":
                return this.createToken(
                    TokenType.WITH,
                    identifier,
                    startLine,
                    startColumn,
                    this.line,
                    this.column,
                );
            default:
                return null;
        }
    }

    /**
     * 检查字面量
     * @returns 词法单元，如果不是字面量则返回null
     */
    private checkLiteral(): Token | null {
        // 检查数字字面量
        const numberToken = this.checkNumberLiteral();
        if (numberToken) {
            return numberToken;
        }

        // 检查字符字面量
        const charToken = this.checkCharacterLiteral();
        if (charToken) {
            return charToken;
        }

        // 检查字符串字面量
        const stringToken = this.checkStringLiteral();
        if (stringToken) {
            return stringToken;
        }

        // 检查原始标识符字面量
        const rawIdentifierToken = this.checkRawIdentifier();
        if (rawIdentifierToken) {
            return rawIdentifierToken;
        }

        return null;
    }

    /**
     * 检查数字字面量
     * @returns 词法单元，如果不是数字字面量则返回null
     */
    private checkNumberLiteral(): Token | null {
        const startPos = this.position;
        const startLine = this.line;
        const startColumn = this.column;

        // 检查整数部分
        if (!this.isDigit(this.source[this.position])) {
            return null;
        }

        // 读取整数部分
        this.position++;
        this.column++;
        while (this.position < this.source.length && this.isDigit(this.source[this.position])) {
            this.position++;
            this.column++;
        }

        // 检查小数部分
        if (this.position < this.source.length && this.source[this.position] === ".") {
            this.position++;
            this.column++;
            if (this.position < this.source.length && this.isDigit(this.source[this.position])) {
                while (
                    this.position < this.source.length &&
                    this.isDigit(this.source[this.position])
                    ) {
                    this.position++;
                    this.column++;
                }
            } else {
                // 小数点后没有数字，回退
                this.position--;
                this.column--;
            }
        }

        const number = this.source.substring(startPos, this.position);
        return this.createToken(
            TokenType.NUMBER_LITERAL,
            number,
            startLine,
            startColumn,
            this.line,
            this.column,
        );
    }

    /**
     * 检查字符是否为数字
     * @param char 字符
     * @returns 是否为数字
     */
    private isDigit(char: string): boolean {
        return /^[0-9]$/.test(char);
    }

    /**
     * 检查字符字面量
     * @returns 词法单元，如果不是字符字面量则返回null
     */
    private checkCharacterLiteral(): Token | null {
        if (this.position >= this.source.length || this.source[this.position] !== "'") {
            return null;
        }

        const startPos = this.position;
        const startLine = this.line;
        const startColumn = this.column;

        // 跳过开始引号
        this.position++;
        this.column++;

        // 读取字符内容
        let charContent = "";
        while (this.position < this.source.length && this.source[this.position] !== "'") {
            if (this.source[this.position] === "\\") {
                // 转义字符
                this.position++;
                this.column++;
                if (this.position < this.source.length) {
                    charContent += this.source[this.position];
                    this.position++;
                    this.column++;
                }
            } else {
                charContent += this.source[this.position];
                this.position++;
                this.column++;
            }
        }

        // 检查结束引号
        if (this.position < this.source.length && this.source[this.position] === "'") {
            this.position++;
            this.column++;
            return this.createToken(
                TokenType.CHARACTER_LITERAL,
                charContent,
                startLine,
                startColumn,
                this.line,
                this.column,
            );
        }

        // 字符字面量未闭合，回退
        this.position = startPos;
        this.line = startLine;
        this.column = startColumn;
        return null;
    }

    /**
     * 检查字符串字面量
     * @returns 词法单元，如果不是字符串字面量则返回null
     */
    private checkStringLiteral(): Token | null {
        if (this.position >= this.source.length || this.source[this.position] !== '"') {
            return null;
        }

        const startPos = this.position;
        const startLine = this.line;
        const startColumn = this.column;

        // 跳过开始引号
        this.position++;
        this.column++;

        // 读取字符串内容
        let stringContent = "";
        while (this.position < this.source.length && this.source[this.position] !== '"') {
            if (this.source[this.position] === "\\") {
                // 转义字符
                this.position++;
                this.column++;
                if (this.position < this.source.length) {
                    stringContent += this.source[this.position];
                    this.position++;
                    this.column++;
                }
            } else {
                stringContent += this.source[this.position];
                this.position++;
                this.column++;
            }
        }

        // 检查结束引号
        if (this.position < this.source.length && this.source[this.position] === '"') {
            this.position++;
            this.column++;
            return this.createToken(
                TokenType.STRING_LITERAL,
                stringContent,
                startLine,
                startColumn,
                this.line,
                this.column,
            );
        }

        // 字符串字面量未闭合，回退
        this.position = startPos;
        this.line = startLine;
        this.column = startColumn;
        return null;
    }

    /**
     * 检查原始标识符字面量
     * @returns 词法单元，如果不是原始标识符字面量则返回null
     */
    private checkRawIdentifier(): Token | null {
        if (this.position >= this.source.length || this.source[this.position] !== "`") {
            return null;
        }

        const startPos = this.position;
        const startLine = this.line;
        const startColumn = this.column;

        // 跳过开始反引号
        this.position++;
        this.column++;

        // 读取原始标识符内容
        let rawContent = "";
        while (this.position < this.source.length && this.source[this.position] !== "`") {
            rawContent += this.source[this.position];
            this.position++;
            this.column++;
        }

        // 检查结束反引号
        if (this.position < this.source.length && this.source[this.position] === "`") {
            this.position++;
            this.column++;
            return this.createToken(
                TokenType.RAW_IDENTIFIER,
                rawContent,
                startLine,
                startColumn,
                this.line,
                this.column,
            );
        }

        // 原始标识符未闭合，回退
        this.position = startPos;
        this.line = startLine;
        this.column = startColumn;
        return null;
    }

    /**
     * 检查运算符和标点符号
     * @returns 词法单元，如果不是运算符或标点符号则返回null
     */
    private checkOperator(): Token | null {
        const startLine = this.line;
        const startColumn = this.column;

        // 检查双字符运算符
        if (this.position + 1 < this.source.length) {
            const twoChar = this.source.substring(this.position, this.position + 2);
            switch (twoChar) {
                case "&&":
                    this.position += 2;
                    this.column += 2;
                    return this.createToken(
                        TokenType.LOGICAL_AND,
                        twoChar,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "||":
                    this.position += 2;
                    this.column += 2;
                    return this.createToken(
                        TokenType.LOGICAL_OR,
                        twoChar,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "==":
                    this.position += 2;
                    this.column += 2;
                    return this.createToken(
                        TokenType.EQUAL,
                        twoChar,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "!=":
                    this.position += 2;
                    this.column += 2;
                    return this.createToken(
                        TokenType.NOT_EQUAL,
                        twoChar,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "<=":
                    this.position += 2;
                    this.column += 2;
                    return this.createToken(
                        TokenType.LESS_THAN_OR_EQUAL,
                        twoChar,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case ">=":
                    this.position += 2;
                    this.column += 2;
                    return this.createToken(
                        TokenType.GREATER_THAN_OR_EQUAL,
                        twoChar,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "..":
                    this.position += 2;
                    this.column += 2;
                    return this.createToken(
                        TokenType.RANGE,
                        twoChar,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "::":
                    this.position += 2;
                    this.column += 2;
                    return this.createToken(
                        TokenType.DOUBLE_COLON,
                        twoChar,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "|>":
                    this.position += 2;
                    this.column += 2;
                    return this.createToken(
                        TokenType.PIPE,
                        twoChar,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
            }
        }

        // 检查单字符运算符和标点符号
        if (this.position < this.source.length) {
            const char = this.source[this.position];
            switch (char) {
                case "=":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.ASSIGN,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "+":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.PLUS,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "-":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.MINUS,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "*":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.MULTIPLY,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "/":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.DIVIDE,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "%":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.MODULO,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "!":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.LOGICAL_NOT,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "<":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.LESS_THAN,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case ">":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.GREATER_THAN,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "(":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.LEFT_PAREN,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case ")":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.RIGHT_PAREN,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "[":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.LEFT_BRACKET,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "]":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.RIGHT_BRACKET,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "{":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.LEFT_BRACE,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case "}":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.RIGHT_BRACE,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case ";":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.SEMICOLON,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case ",":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.COMMA,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case ".":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.DOT,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
                case ":":
                    this.position++;
                    this.column++;
                    return this.createToken(
                        TokenType.COLON,
                        char,
                        startLine,
                        startColumn,
                        this.line,
                        this.column,
                    );
            }
        }

        return null;
    }

    /**
     * 检查文本
     * @returns 词法单元，如果不是文本则返回null
     */
    private checkText(): Token | null {
        const startPos = this.position;
        const startLine = this.line;
        const startColumn = this.column;

        while (this.position < this.source.length) {
            if (
                this.source.startsWith(this.delimiters.interpolate[0], this.position) ||
                this.source.startsWith(this.delimiters.comment[0], this.position) ||
                this.source.startsWith(this.delimiters.interpolate[1], this.position) ||
                this.source.startsWith(this.delimiters.comment[1], this.position)
            ) {
                break;
            }

            if (this.source[this.position] === "\n") {
                this.position++;
                this.line++;
                this.column = 1;
            } else {
                this.position++;
                this.column++;
            }
        }

        if (this.position > startPos) {
            const text = this.source.substring(startPos, this.position);
            return this.createToken(
                TokenType.TEXT,
                text,
                startLine,
                startColumn,
                this.line,
                this.column,
            );
        }

        return null;
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
    private createToken(
        type: TokenType,
        value: string,
        startLine: number,
        startColumn: number,
        endLine: number,
        endColumn: number,
    ): Token {
        return createToken(type, value, startLine, startColumn, endLine, endColumn);
    }
}

/**
 * 创建词法分析器
 * @param source 模板源代码
 * @param language 语言配置
 * @returns 词法分析器实例
 */
export function createLexer(source: string, language?: DejavuLanguage): Lexer {
    return new Lexer(source, language);
}
