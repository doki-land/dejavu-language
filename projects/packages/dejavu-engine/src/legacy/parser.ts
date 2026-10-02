/**
 * DejaVu 模板引擎语法分析器
 */

import { Token, TokenType } from "./token";
import { Lexer, createLexer } from "./lexer";
import {
    Program,
    Statement,
    Expression,
    Identifier,
    Literal,
    CallExpression,
    ArrayExpression,
    TupleExpression,
    ObjectExpression,
    ObjectProperty,
    IfStatement,
    ElseIfClause,
    ForStatement,
    ForInStatement,
    WhileStatement,
    MatchStatement,
    MatchCase,
    NamespaceDeclaration,
    UsingDeclaration,
    BlockStatement,
    BreakStatement,
    ContinueStatement,
    ExtendsStatement,
    BlockDeclaration,
    PrependStatement,
    AppendStatement,
    SuperExpression,
    VariableDeclaration,
    FunctionDeclaration,
    AssignmentStatement,
    ExpressionStatement,
    Text,
    Comment,
    RangeExpression,
    AsExpression,
    TypedParameter,
    TypeAnnotation,
    ArrayTypeAnnotation,
    TupleTypeAnnotation,
    PipeExpression,
    IncludeStatement,
    Node,
    createProgram,
    createIdentifier,
    createLiteral,
    createBinaryExpression,
    createUnaryExpression,
    createCallExpression,
    createMemberExpression,
    createArrayExpression,
    createTupleExpression,
    createObjectExpression,
    createObjectProperty,
    createIfStatement,
    createElseIfClause,
    createForStatement,
    createForInStatement,
    createWhileStatement,
    createMatchStatement,
    createMatchCase,
    createNamespaceDeclaration,
    createUsingDeclaration,
    createBlockStatement,
    createBreakStatement,
    createContinueStatement,
    createExtendsStatement,
    createBlockDeclaration,
    createPrependStatement,
    createAppendStatement,
    createSuperExpression,
    createVariableDeclaration,
    createFunctionDeclaration,
    createAssignmentStatement,
    createExpressionStatement,
    createText,
    createComment,
    createRangeExpression,
    createAsExpression,
    createTypedParameter,
    createArrayTypeAnnotation,
    createTupleTypeAnnotation,
    createPipeExpression,
    createIncludeStatement,
} from "./ast";
import { DejavuLanguage, language as defaultLanguage } from "./language";
import { ParseError, SourceRange, createError, ErrorCode, ErrorSuggestion } from "./error-types";

type BlockBody = (Statement | Expression | Text | Comment)[];

/**
 * 语法分析器类
 */
export class Parser {
    /** 词法分析器 */
    private lexer: Lexer;
    /** 当前词法单元 */
    private currentToken: Token;
    /** token 缓存（用于回溯） */
    private tokenBuffer: Token[] = [];
    /** 缓存索引（用于回溯） */
    private bufferIndex: number = -1;
    /** 是否在模板模式 */
    private inTemplateMode: boolean;
    /** 源代码 */
    private readonly source: string;
    /** 文件路径（用于错误报告） */
    private filePath?: string;

    /**
     * 构造函数
     * @param source 模板源代码
     * @param language 语言配置
     * @param filePath 文件路径（可选，用于错误报告）
     */
    constructor(source: string, language?: DejavuLanguage, filePath?: string) {
        const lang = language || defaultLanguage;
        this.source = source;
        this.filePath = filePath;
        this.lexer = createLexer(source, lang);
        this.currentToken = this.lexer.nextToken();
        this.inTemplateMode = true;
    }

    /**
     * 埥看下一个 token（不前进）
     * @returns 下一个 token
     */
    private peekToken(): Token {
        return this.lexer.nextToken();
    }

    /**
     * 停止查看并前进
     */
    private advance(): void {
        if (this.bufferIndex >= 0 && this.bufferIndex < this.tokenBuffer.length - 1) {
            this.currentToken = this.tokenBuffer[++this.bufferIndex];
        } else {
            this.tokenBuffer.push(this.currentToken);
            this.bufferIndex = this.tokenBuffer.length - 1;
            this.currentToken = this.lexer.nextToken();
        }
    }

    /**
     * 查看下一个 token（不前进）
     * @returns 下一个 token
     */
    private peekNextToken(): Token {
        return this.lexer.peekToken();
    }

    /**
     * 开始缓存 token
     */
    private startBuffering(): void {
        this.tokenBuffer = [];
        this.bufferIndex = -1;
    }

    /**
     * 回退到上一个 token
     */
    private backtrack(): void {
        if (this.tokenBuffer.length > 0) {
            this.currentToken = this.tokenBuffer[0];
            this.bufferIndex = 0;
        }
    }

    /**
     * 解析模板，生成AST
     * @returns 程序节点
     */
    parse(): Program {
        const body: (Statement | Expression | Text | Comment)[] = [];

        while (!this.isToken(TokenType.EOF)) {
            if (this.isToken(TokenType.TEXT)) {
                body.push(this.parseText());
            } else if (this.isToken(TokenType.COMMENT)) {
                body.push(this.parseComment());
            } else if (this.isToken(TokenType.DELIMITER_START)) {
                const node = this.parseTemplateExpression();
                if (node !== null) {
                    body.push(node);
                }
            } else if (
                this.isToken(TokenType.END) ||
                this.isToken(TokenType.ELSE) ||
                this.isToken(TokenType.CASE)
            ) {
                break;
            } else {
                break;
            }
        }

        return createProgram(body);
    }

    /**
     * 解析文本节点
     * @returns 文本节点
     */
    private parseText(): Text {
        const token = this.currentToken;
        this.advance();
        return createText(token.value, token.loc);
    }

    /**
     * 解析注释节点
     * @returns 注释节点
     */
    private parseComment(): Comment {
        const token = this.currentToken;
        this.advance();
        return createComment(token.value, true, token.loc);
    }

    /**
     * 解析模板表达式
     * @returns 语句或表达式节点，如果遇到结束标记则返回null
     */
    private parseTemplateExpression(): Statement | Expression | null {
        this.advance();

        if (
            this.isToken(TokenType.END) ||
            this.isToken(TokenType.ELSE) ||
            this.isToken(TokenType.CASE)
        ) {
            return null;
        }

        const statement = this.parseTemplateStatement();

        if (statement === null) {
            return null;
        }

        if (this.isToken(TokenType.DELIMITER_END)) {
            this.advance();
        }

        return statement;
    }

    /**
     * 解析模板内的语句
     * @returns 语句或表达式节点
     */
    private parseTemplateStatement(): Statement | Expression | null {
        const type = this.currentToken.type;
        switch (type) {
            case TokenType.END:
            case TokenType.ELSE:
            case TokenType.CASE:
                return null;
            case TokenType.LET:
                return this.parseVariableDeclaration();
            case TokenType.MICRO:
                return this.parseMicroDeclaration();
            case TokenType.NAMESPACE:
                return this.parseNamespaceDeclaration();
            case TokenType.USING:
                return this.parseUsingDeclaration();
            case TokenType.IF:
                return this.parseTemplateIfStatement();
            case TokenType.LOOP:
                return this.parseTemplateLoopStatement();
            case TokenType.MATCH:
                return this.parseTemplateMatchStatement();
            case TokenType.BREAK:
                return this.parseBreakStatement();
            case TokenType.CONTINUE:
                return this.parseContinueStatement();
            case TokenType.EXTENDS:
                return this.parseExtendsStatement();
            case TokenType.BLOCK:
                return this.parseBlockDeclaration();
            case TokenType.PREPEND:
                return this.parsePrependStatement();
            case TokenType.APPEND:
                return this.parseAppendStatement();
            case TokenType.IDENTIFIER:
                return this.parseTemplateExpressionOrAssignment();
            case TokenType.NUMBER_LITERAL:
            case TokenType.STRING_LITERAL:
            case TokenType.BOOLEAN_LITERAL:
            case TokenType.CHARACTER_LITERAL:
            case TokenType.LEFT_PAREN:
            case TokenType.LEFT_BRACKET:
            case TokenType.LEFT_BRACE:
                return this.parseExpressionStatement();
            case TokenType.DELIMITER_END:
                return null;
            default:
                return null;
        }
    }

    /**
     * 解析模板内的表达式或赋值语句
     * @returns 表达式或赋值语句节点，如果遇到结束标记则返回null
     */
    private parseTemplateExpressionOrAssignment(): Statement | Expression | null {
        if (
            this.isToken(TokenType.END) ||
            this.isToken(TokenType.ELSE) ||
            this.isToken(TokenType.CASE) ||
            this.isToken(TokenType.DELIMITER_END)
        ) {
            return null;
        }

        const startLoc = this.currentToken.loc;
        const left = this.parseExpression();

        if (this.isToken(TokenType.ASSIGN)) {
            this.advance();
            const right = this.parseExpression();
            this.consumeOptionalSemicolon();
            return createAssignmentStatement(left, right, this.createLoc(startLoc));
        }

        this.consumeOptionalSemicolon();
        return createExpressionStatement(left, this.createLoc(startLoc));
    }

    /**
     * 解析模板内的if语句（支持else if和end if）
     * @returns if语句节点
     */
    private parseTemplateIfStatement(): IfStatement {
        const startLoc = this.currentToken.loc;
        this.advance();

        const test = this.parseExpression();
        const consequent = this.parseTemplateBlock(["else", "end"]);

        const elseIfs: ElseIfClause[] = [];
        let alternate: Statement | undefined;

        while (true) {
            if (this.isToken(TokenType.ELSE)) {
                this.advance();

                if (this.isToken(TokenType.IF)) {
                    this.advance();
                    const elseIfTest = this.parseExpression();
                    const elseIfConsequent = this.parseTemplateBlock(["else", "end"]);
                    elseIfs.push(createElseIfClause(elseIfTest, elseIfConsequent));
                } else {
                    alternate = this.parseTemplateBlock(["end"]);
                    break;
                }
            } else if (this.isToken(TokenType.END)) {
                this.advance();
                if (this.isToken(TokenType.IF)) {
                    this.advance();
                }
                break;
            } else if (this.isToken(TokenType.DELIMITER_START)) {
                this.advance();
                if (this.isToken(TokenType.ELSE)) {
                    this.advance();

                    if (this.isToken(TokenType.IF)) {
                        this.advance();
                        const elseIfTest = this.parseExpression();
                        const elseIfConsequent = this.parseTemplateBlock(["else", "end"]);
                        elseIfs.push(createElseIfClause(elseIfTest, elseIfConsequent));
                    } else {
                        alternate = this.parseTemplateBlock(["end"]);
                        break;
                    }
                } else if (this.isToken(TokenType.END)) {
                    this.advance();
                    if (this.isToken(TokenType.IF)) {
                        this.advance();
                    }
                    break;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if (this.isToken(TokenType.DELIMITER_END)) {
            this.advance();
        }

        return createIfStatement(
            test,
            consequent,
            elseIfs.length > 0 ? elseIfs : undefined,
            alternate,
            this.createLoc(startLoc),
        );
    }

    /**
     * 解析模板内的循环语句（支持loop item in list和end loop）
     * @returns 循环语句节点
     */
    private parseTemplateLoopStatement(): ForStatement | ForInStatement | WhileStatement {
        const startLoc = this.currentToken.loc;
        this.advance();

        return this.parseForInLoop(startLoc);
    }

    /**
     * 解析传统for循环
     * @param startLoc 起始位置
     * @returns for循环语句节点
     */
    private parseTraditionalForLoop(startLoc: Token["loc"]): ForStatement {
        this.advance();

        let init: VariableDeclaration | AssignmentStatement | Expression | null = null;
        if (!this.isToken(TokenType.SEMICOLON)) {
            if (this.isToken(TokenType.LET)) {
                init = this.parseVariableDeclaration();
            } else {
                init = this.parseExpression();
            }
        }

        this.expectAndConsume(TokenType.SEMICOLON, ";");

        let test: Expression | null = null;
        if (!this.isToken(TokenType.SEMICOLON)) {
            test = this.parseExpression();
        }

        this.expectAndConsume(TokenType.SEMICOLON, ";");

        let update: Expression | null = null;
        if (!this.isToken(TokenType.RIGHT_PAREN)) {
            update = this.parseExpression();
        }

        this.expectAndConsume(TokenType.RIGHT_PAREN, ")");

        const body = this.parseTemplateBlock(["end"]);
        this.skipEndMarker("loop");

        return createForStatement(init, test, update, body, this.createLoc(startLoc));
    }

    /**
     * 解析for-in遍历循环
     * @param startLoc 起始位置
     * @returns for-in循环语句节点
     */
    private parseForInLoop(startLoc: Token["loc"]): ForInStatement {
        let left: Identifier | [Identifier, Identifier];

        if (this.isToken(TokenType.LEFT_PAREN)) {
            this.advance();

            const first = this.parseIdentifierNode();
            left = first;

            if (this.isToken(TokenType.COMMA)) {
                this.advance();
                const second = this.parseIdentifierNode();
                left = [first, second];
            }

            this.expectAndConsume(TokenType.RIGHT_PAREN, ")");
        } else if (this.isToken(TokenType.IDENTIFIER)) {
            left = this.parseIdentifierNode();
        } else if (this.isToken(TokenType.END) || this.isToken(TokenType.CASE)) {
            this.throwError(
                ErrorCode.E008_INVALID_EXPRESSION,
                { detail: `期望标识符或 ( 在循环语句中，但发现 ${this.currentToken.type}` },
                this.currentToken.loc,
            );
        }

        this.expectAndConsume(TokenType.IN, "in");

        const right = this.parseExpression();
        const body = this.parseTemplateBlock(["end"]);
        this.skipEndMarker("loop");

        return createForInStatement(left, right, body, this.createLoc(startLoc));
    }

    /**
     * 解析模板内的match语句
     * @returns match语句节点
     */
    private parseTemplateMatchStatement(): MatchStatement {
        const startLoc = this.currentToken.loc;
        this.advance();

        const discriminant = this.parseExpression();
        const cases: MatchCase[] = [];

        while (true) {
            if (this.isToken(TokenType.CASE)) {
                this.advance();

                let pattern: Expression | null = null;
                if (this.isToken(TokenType.IDENTIFIER) && this.currentToken.value === "_") {
                    this.advance();
                    pattern = null;
                } else {
                    pattern = this.parseExpression();
                }

                const consequent = this.parseTemplateBlock(["case", "end"]);
                cases.push(createMatchCase(pattern, consequent));
            } else if (this.isToken(TokenType.END)) {
                this.advance();
                if (this.isToken(TokenType.MATCH)) {
                    this.advance();
                }
                break;
            } else if (this.isToken(TokenType.DELIMITER_START)) {
                this.advance();
                if (this.isToken(TokenType.CASE)) {
                    this.advance();

                    let pattern: Expression | null = null;
                    if (this.isToken(TokenType.IDENTIFIER) && this.currentToken.value === "_") {
                        this.advance();
                        pattern = null;
                    } else {
                        pattern = this.parseExpression();
                    }

                    const consequent = this.parseTemplateBlock(["case", "end"]);
                    cases.push(createMatchCase(pattern, consequent));
                } else if (this.isToken(TokenType.END)) {
                    this.advance();
                    if (this.isToken(TokenType.MATCH)) {
                        this.advance();
                    }
                    break;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if (this.isToken(TokenType.DELIMITER_END)) {
            this.advance();
        }

        return createMatchStatement(discriminant, cases, this.createLoc(startLoc));
    }

    /**
     * 解析微函数声明（支持end micro）
     * @returns 函数声明节点
     */
    private parseMicroDeclaration(): FunctionDeclaration {
        const startLoc = this.currentToken.loc;
        this.advance();

        const name = this.parseIdentifierNode();
        this.expectAndConsume(TokenType.LEFT_PAREN, "(");

        const params: TypedParameter[] = [];
        while (!this.isToken(TokenType.RIGHT_PAREN)) {
            const param = this.parseTypedParameter();
            params.push(param);

            if (this.isToken(TokenType.COMMA)) {
                this.advance();
            } else if (!this.isToken(TokenType.RIGHT_PAREN)) {
                this.throwError(
                    ErrorCode.E002_EXPECTED_TOKEN,
                    { expected: ", 或 )", found: this.currentToken.type },
                    this.currentToken.loc,
                );
            }
        }

        this.advance();

        const body = this.parseTemplateBlock(["end"]);
        this.skipEndMarker("micro");

        const paramIdentifiers = params.map((p) => p.name);
        return createFunctionDeclaration(
            name,
            paramIdentifiers,
            body as BlockStatement,
            this.createLoc(startLoc),
        );
    }

    /**
     * 解析带类型注解的参数
     * @returns 带类型注解的参数节点
     */
    private parseTypedParameter(): TypedParameter {
        const name = this.parseIdentifierNode();

        let typeAnnotation: Identifier | undefined;
        if (this.isToken(TokenType.COLON)) {
            this.advance();
            typeAnnotation = this.parseIdentifierNode();
        }

        return createTypedParameter(name, typeAnnotation);
    }

    /**
     * 解析命名空间声明
     * @returns 命名空间声明节点
     */
    private parseNamespaceDeclaration(): NamespaceDeclaration {
        const startLoc = this.currentToken.loc;
        this.advance();

        const name = this.parseIdentifierNode();
        this.consumeOptionalSemicolon();

        return createNamespaceDeclaration(name, this.createLoc(startLoc));
    }

    /**
     * 解析using导入声明
     * @returns using声明节点
     */
    private parseUsingDeclaration(): UsingDeclaration {
        const startLoc = this.currentToken.loc;
        this.advance();

        const modulePath: Identifier[] = [];
        modulePath.push(this.parseIdentifierNode());

        while (this.isToken(TokenType.DOUBLE_COLON)) {
            this.advance();
            modulePath.push(this.parseIdentifierNode());
        }

        this.consumeOptionalSemicolon();

        return createUsingDeclaration(modulePath, this.createLoc(startLoc));
    }

    /**
     * 解析模板块（直到遇到指定的结束关键字）
     * @param endKeywords 结束关键字列表
     * @returns 块语句节点
     */
    private parseTemplateBlock(endKeywords: string[]): BlockStatement {
        const body: BlockBody = [];

        while (!this.isToken(TokenType.EOF)) {
            if (this.isToken(TokenType.TEXT)) {
                body.push(this.parseText());
            } else if (this.isToken(TokenType.COMMENT)) {
                body.push(this.parseComment());
            } else if (this.isToken(TokenType.DELIMITER_END)) {
                this.advance();
            } else if (this.isToken(TokenType.DELIMITER_START)) {
                const nextToken = this.peekNextToken();

                if (
                    nextToken.type === TokenType.END ||
                    nextToken.type === TokenType.ELSE ||
                    nextToken.type === TokenType.CASE
                ) {
                    break;
                }

                this.advance();
                const statement = this.parseTemplateStatement();
                if (statement !== null) {
                    body.push(statement);
                    if (this.isToken(TokenType.DELIMITER_END)) {
                        this.advance();
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        return createBlockStatement(body);
    }

    /**
     * 恢复前一个 token（用于回溯）
     * @param type 要恢复的 token 类型
     */
    private restoreToken(type: TokenType): void {
        this.currentToken = {
            type,
            value: "",
            loc: this.currentToken.loc,
        };
    }

    /**
     * 将文本节点转换为注释节点（用于模板块内）
     */
    private parseTextAsComment(): Comment {
        const token = this.currentToken;
        this.advance();
        return createComment(token.value, false, token.loc);
    }

    /**
     * 跳过结束标记
     * @param keyword 关键字名称
     */
    private skipEndMarker(keyword: string): void {
        if (this.isToken(TokenType.END)) {
            this.advance();
        }

        if (this.isToken(TokenType.IDENTIFIER) && this.currentToken.value === keyword) {
            this.advance();
        } else if (
            (keyword === "loop" && this.isToken(TokenType.LOOP)) ||
            (keyword === "match" && this.isToken(TokenType.MATCH)) ||
            (keyword === "micro" && this.isToken(TokenType.MICRO))
        ) {
            this.advance();
        }

        if (this.isToken(TokenType.DELIMITER_END)) {
            this.advance();
        }
    }

    /**
     * 解析break语句
     * @returns break语句节点
     */
    private parseBreakStatement(): BreakStatement {
        const startLoc = this.currentToken.loc;
        this.advance();
        this.consumeOptionalSemicolon();

        return createBreakStatement(this.createLoc(startLoc));
    }

    /**
     * 解析continue语句
     * @returns continue语句节点
     */
    private parseContinueStatement(): ContinueStatement {
        const startLoc = this.currentToken.loc;
        this.advance();
        this.consumeOptionalSemicolon();

        return createContinueStatement(this.createLoc(startLoc));
    }

    /**
     * 解析extends语句
     * @returns extends语句节点
     */
    private parseExtendsStatement(): ExtendsStatement {
        const startLoc = this.currentToken.loc;
        this.advance();

        const template = this.parseStringLiteral();

        return createExtendsStatement(template, this.createLoc(startLoc));
    }

    /**
     * 解析block声明
     * @returns block声明节点
     */
    private parseBlockDeclaration(): BlockDeclaration {
        const startLoc = this.currentToken.loc;
        this.advance();

        const name = this.parseIdentifierNode();
        const block = this.parseTemplateBlock(["end"]);

        if (this.isToken(TokenType.END)) {
            this.advance();
            if (this.isToken(TokenType.BLOCK)) {
                this.advance();
            }
        }

        return createBlockDeclaration(name, block.body, this.createLoc(startLoc));
    }

    /**
     * 解析prepend语句
     * @returns prepend语句节点
     */
    private parsePrependStatement(): PrependStatement {
        const startLoc = this.currentToken.loc;
        this.advance();

        const name = this.parseIdentifierNode();
        const block = this.parseTemplateBlock(["end"]);

        if (this.isToken(TokenType.END)) {
            this.advance();
            if (this.isToken(TokenType.BLOCK)) {
                this.advance();
            }
        }

        return createPrependStatement(name, block.body, this.createLoc(startLoc));
    }

    /**
     * 解析append语句
     * @returns append语句节点
     */
    private parseAppendStatement(): AppendStatement {
        const startLoc = this.currentToken.loc;
        this.advance();

        const name = this.parseIdentifierNode();
        const block = this.parseTemplateBlock(["end"]);

        if (this.isToken(TokenType.END)) {
            this.advance();
            if (this.isToken(TokenType.BLOCK)) {
                this.advance();
            }
        }

        return createAppendStatement(name, block.body, this.createLoc(startLoc));
    }

    /**
     * 解析 include 语句
     * @returns include 语句节点
     */
    private parseIncludeStatement(): IncludeStatement {
        const startLoc = this.currentToken.loc;
        this.advance();

        const template = this.parseExpression();

        let context: Expression | undefined;
        if (this.isToken(TokenType.WITH)) {
            this.advance();
            context = this.parseExpression();
        }

        return createIncludeStatement(template, context, this.createLoc(startLoc));
    }

    /**
     * 解析语句
     * @returns 语句节点
     */
    private parseStatement(): Statement | Expression {
        const type = this.currentToken.type;
        switch (type) {
            case TokenType.LET:
                return this.parseVariableDeclaration();
            case TokenType.MICRO:
                return this.parseFunctionDeclaration();
            case TokenType.IF:
                return this.parseIfStatement();
            case TokenType.LOOP:
                return this.parseLoopStatement();
            case TokenType.LEFT_BRACE:
                return this.parseBlockStatement();
            default:
                if (this.isToken(TokenType.IDENTIFIER)) {
                    const nextToken = this.lexer.nextToken();
                    if (nextToken.type === TokenType.ASSIGN) {
                        return this.parseAssignmentStatement();
                    }
                }
                return this.parseExpressionStatement();
        }
    }

    /**
     * 解析变量声明
     * @returns 变量声明节点
     */
    private parseVariableDeclaration(): VariableDeclaration {
        const startLoc = this.currentToken.loc;
        this.advance();

        const name = this.parseIdentifierNode();

        let typeAnnotation: TypeAnnotation | undefined;
        if (this.isToken(TokenType.COLON)) {
            this.advance();
            typeAnnotation = this.parseTypeAnnotation();
        }

        let init: Expression | undefined;
        if (this.isToken(TokenType.ASSIGN)) {
            this.advance();
            init = this.parseExpression();
        }

        this.consumeOptionalSemicolon();

        return createVariableDeclaration(name, init, typeAnnotation, this.createLoc(startLoc));
    }

    /**
     * 解析类型注解
     * @returns 类型注解节点
     */
    private parseTypeAnnotation(): TypeAnnotation {
        if (this.isToken(TokenType.LEFT_BRACKET)) {
            return this.parseArrayTypeAnnotation();
        }

        if (this.isToken(TokenType.LEFT_PAREN)) {
            return this.parseTupleTypeAnnotation();
        }

        return this.parseIdentifierNode();
    }

    /**
     * 解析数组类型注解 [Type; N]
     * @returns 数组类型注解节点
     */
    private parseArrayTypeAnnotation(): ArrayTypeAnnotation {
        const startLoc = this.currentToken.loc;
        this.advance();

        const elementType = this.parseTypeAnnotation();

        let size: Literal | undefined;
        if (this.isToken(TokenType.SEMICOLON)) {
            this.advance();
            if (this.isToken(TokenType.NUMBER_LITERAL)) {
                size = this.parseNumberLiteral();
            }
        }

        this.expectAndConsume(TokenType.RIGHT_BRACKET, "]");

        return createArrayTypeAnnotation(elementType, size, this.createLoc(startLoc));
    }

    /**
     * 解析元组类型注解 (Type1, Type2, ...)
     * @returns 元组类型注解节点
     */
    private parseTupleTypeAnnotation(): TupleTypeAnnotation {
        const startLoc = this.currentToken.loc;
        this.advance();

        const elementTypes: TypeAnnotation[] = [];
        while (!this.isToken(TokenType.RIGHT_PAREN)) {
            elementTypes.push(this.parseTypeAnnotation());

            if (this.isToken(TokenType.COMMA)) {
                this.advance();
            } else if (!this.isToken(TokenType.RIGHT_PAREN)) {
                this.throwError(
                    ErrorCode.E002_EXPECTED_TOKEN,
                    { expected: ", 或 )", found: this.currentToken.type },
                    this.currentToken.loc,
                );
            }
        }

        this.advance();

        return createTupleTypeAnnotation(elementTypes, this.createLoc(startLoc));
    }

    /**
     * 解析函数声明（传统模式）
     * @returns 函数声明节点
     */
    private parseFunctionDeclaration(): FunctionDeclaration {
        const startLoc = this.currentToken.loc;
        this.advance();

        const name = this.parseIdentifierNode();
        this.expectAndConsume(TokenType.LEFT_PAREN, "(");

        const params: Identifier[] = [];
        while (!this.isToken(TokenType.RIGHT_PAREN)) {
            if (this.isToken(TokenType.IDENTIFIER)) {
                params.push(this.parseIdentifierNode());

                if (this.isToken(TokenType.COLON)) {
                    this.advance();
                    this.parseIdentifierNode();
                }

                if (this.isToken(TokenType.COMMA)) {
                    this.advance();
                }
            } else {
                this.throwError(
                    ErrorCode.E002_EXPECTED_TOKEN,
                    { expected: "标识符 或 )", found: this.currentToken.type },
                    this.currentToken.loc,
                );
            }
        }

        this.advance();

        const body = this.parseBlockStatement();

        return createFunctionDeclaration(name, params, body, this.createLoc(startLoc));
    }

    /**
     * 解析if语句（传统模式）
     * @returns if语句节点
     */
    private parseIfStatement(): IfStatement {
        const startLoc = this.currentToken.loc;
        this.advance();

        this.expectAndConsume(TokenType.LEFT_PAREN, "(");
        const test = this.parseExpression();
        this.expectAndConsume(TokenType.RIGHT_PAREN, ")");

        const consequent = this.parseStatement() as Statement;

        const elseIfs: ElseIfClause[] = [];
        let alternate: Statement | undefined;

        while (this.isToken(TokenType.ELSE)) {
            this.advance();

            if (this.isToken(TokenType.IF)) {
                this.advance();
                this.expectAndConsume(TokenType.LEFT_PAREN, "(");
                const elseIfTest = this.parseExpression();
                this.expectAndConsume(TokenType.RIGHT_PAREN, ")");
                const elseIfConsequent = this.parseStatement() as Statement;
                elseIfs.push(createElseIfClause(elseIfTest, elseIfConsequent));
            } else {
                alternate = this.parseStatement() as Statement;
                break;
            }
        }

        return createIfStatement(
            test,
            consequent,
            elseIfs.length > 0 ? elseIfs : undefined,
            alternate,
            this.createLoc(startLoc),
        );
    }

    /**
     * 解析循环语句（传统模式）
     * @returns 循环语句节点
     */
    private parseLoopStatement(): ForStatement | WhileStatement {
        const startLoc = this.currentToken.loc;
        this.advance();

        if (this.isToken(TokenType.LEFT_PAREN)) {
            this.advance();

            let init: VariableDeclaration | AssignmentStatement | Expression | null = null;
            if (!this.isToken(TokenType.SEMICOLON)) {
                if (this.isToken(TokenType.LET)) {
                    init = this.parseVariableDeclaration();
                } else {
                    init = this.parseExpression();
                }
            }

            this.expectAndConsume(TokenType.SEMICOLON, ";");

            let test: Expression | null = null;
            if (!this.isToken(TokenType.SEMICOLON)) {
                test = this.parseExpression();
            }

            this.expectAndConsume(TokenType.SEMICOLON, ";");

            let update: Expression | null = null;
            if (!this.isToken(TokenType.RIGHT_PAREN)) {
                update = this.parseExpression();
            }

            this.expectAndConsume(TokenType.RIGHT_PAREN, ")");

            const body = this.parseStatement() as Statement;

            return createForStatement(init, test, update, body, this.createLoc(startLoc));
        } else {
            const test = this.parseExpression();
            const body = this.parseStatement() as Statement;

            return createWhileStatement(test, body, this.createLoc(startLoc));
        }
    }

    /**
     * 解析块语句
     * @returns 块语句节点
     */
    private parseBlockStatement(): BlockStatement {
        const startLoc = this.currentToken.loc;
        this.advance();

        const body: BlockBody = [];
        while (!this.isToken(TokenType.RIGHT_BRACE) && !this.isToken(TokenType.EOF)) {
            if (this.isToken(TokenType.COMMENT)) {
                body.push(this.parseComment());
            } else {
                body.push(this.parseStatement());
            }
        }

        if (this.isToken(TokenType.RIGHT_BRACE)) {
            this.advance();
        }

        return createBlockStatement(body, this.createLoc(startLoc));
    }

    /**
     * 解析赋值语句
     * @returns 赋值语句节点
     */
    private parseAssignmentStatement(): AssignmentStatement {
        const startLoc = this.currentToken.loc;
        const left = this.parseExpression();

        this.expectAndConsume(TokenType.ASSIGN, "=");

        const right = this.parseExpression();
        this.consumeOptionalSemicolon();

        return createAssignmentStatement(left, right, this.createLoc(startLoc));
    }

    /**
     * 解析表达式语句
     * @returns 表达式语句节点
     */
    private parseExpressionStatement(): ExpressionStatement {
        const startLoc = this.currentToken.loc;
        const expression = this.parseExpression();
        this.consumeOptionalSemicolon();

        return createExpressionStatement(expression, this.createLoc(startLoc));
    }

    /**
     * 解析表达式
     * @returns 表达式节点
     */
    private parseExpression(): Expression {
        return this.parsePipeExpression();
    }

    /**
     * 解析逻辑或表达式
     * @returns 表达式节点
     */
    private parseLogicalOrExpression(): Expression {
        let left = this.parseLogicalAndExpression();

        while (this.isToken(TokenType.LOGICAL_OR)) {
            const operator = this.currentToken.value;
            this.advance();
            const right = this.parseLogicalAndExpression();
            left = createBinaryExpression(operator, left, right);
        }

        return left;
    }

    /**
     * 解析逻辑与表达式
     * @returns 表达式节点
     */
    private parseLogicalAndExpression(): Expression {
        let left = this.parseComparisonExpression();

        while (this.isToken(TokenType.LOGICAL_AND)) {
            const operator = this.currentToken.value;
            this.advance();
            const right = this.parseComparisonExpression();
            left = createBinaryExpression(operator, left, right);
        }

        return left;
    }

    /**
     * 解析比较表达式
     * @returns 表达式节点
     */
    private parseComparisonExpression(): Expression {
        let left = this.parseAdditiveExpression();

        while (this.isComparisonOperator(this.currentToken.type)) {
            const operator = this.currentToken.value;
            this.advance();
            const right = this.parseAdditiveExpression();
            left = createBinaryExpression(operator, left, right);
        }

        return left;
    }

    /**
     * 检查是否为比较运算符
     */
    private isComparisonOperator(type: TokenType): boolean {
        return [
            TokenType.EQUAL,
            TokenType.NOT_EQUAL,
            TokenType.LESS_THAN,
            TokenType.GREATER_THAN,
            TokenType.LESS_THAN_OR_EQUAL,
            TokenType.GREATER_THAN_OR_EQUAL,
        ].includes(type);
    }

    /**
     * 解析加法表达式
     * @returns 表达式节点
     */
    private parseAdditiveExpression(): Expression {
        let left = this.parseMultiplicativeExpression();

        while (this.isToken(TokenType.PLUS) || this.isToken(TokenType.MINUS)) {
            const operator = this.currentToken.value;
            this.advance();
            const right = this.parseMultiplicativeExpression();
            left = createBinaryExpression(operator, left, right);
        }

        return left;
    }

    /**
     * 解析乘法表达式
     * @returns 表达式节点
     */
    private parseMultiplicativeExpression(): Expression {
        let left = this.parseRangeExpression();

        while (
            this.isToken(TokenType.MULTIPLY) ||
            this.isToken(TokenType.DIVIDE) ||
            this.isToken(TokenType.MODULO)
        ) {
            const operator = this.currentToken.value;
            this.advance();
            const right = this.parseRangeExpression();
            left = createBinaryExpression(operator, left, right);
        }

        return left;
    }

    /**
     * 解析范围表达式
     * @returns 表达式节点
     */
    private parseRangeExpression(): Expression {
        let left = this.parseUnaryExpression();

        if (this.isToken(TokenType.RANGE)) {
            this.advance();
            const right = this.parseUnaryExpression();
            left = createRangeExpression(left, right);
        }

        return left;
    }

    /**
     * 解析类型转换表达式
     * @returns 表达式节点
     */
    private parseAsExpression(): Expression {
        let left = this.parseLogicalOrExpression();

        while (this.isToken(TokenType.AS)) {
            this.advance();
            const typeAnnotation = this.parseIdentifierNode();
            left = createAsExpression(left, typeAnnotation);
        }

        return left;
    }

    /**
     * 解析管道表达式
     * @returns 表达式节点
     */
    private parsePipeExpression(): Expression {
        let left = this.parseAsExpression();

        while (this.isToken(TokenType.PIPE)) {
            this.advance();

            const filterName = this.parseIdentifierNode();

            const args: Expression[] = [];
            if (this.isToken(TokenType.COLON)) {
                this.advance();
                args.push(this.parseAsExpression());

                while (this.isToken(TokenType.COMMA)) {
                    this.advance();
                    args.push(this.parseAsExpression());
                }
            }

            left = createPipeExpression(left, filterName, args);
        }

        return left;
    }

    /**
     * 解析一元表达式
     * @returns 表达式节点
     */
    private parseUnaryExpression(): Expression {
        if (this.isToken(TokenType.LOGICAL_NOT) || this.isToken(TokenType.MINUS)) {
            const operator = this.currentToken.value;
            this.advance();
            const argument = this.parseUnaryExpression();
            return createUnaryExpression(operator, argument, true);
        }

        return this.parsePrimaryExpression();
    }

    /**
     * 解析 primary 表达式
     * @returns 表达式节点
     */
    private parsePrimaryExpression(): Expression {
        const type = this.currentToken.type;
        switch (type) {
            case TokenType.IDENTIFIER:
                return this.parseIdentifierExpression();
            case TokenType.NUMBER_LITERAL:
                return this.parseNumberLiteral();
            case TokenType.STRING_LITERAL:
                return this.parseStringLiteral();
            case TokenType.BOOLEAN_LITERAL:
                return this.parseBooleanLiteral();
            case TokenType.CHARACTER_LITERAL:
                return this.parseCharacterLiteral();
            case TokenType.LEFT_PAREN:
                return this.parseParenthesizedExpression();
            case TokenType.LEFT_BRACKET:
                return this.parseArrayExpression();
            case TokenType.LEFT_BRACE:
                return this.parseObjectExpression();
            case TokenType.SUPER:
                return this.parseSuperExpression();
            default:
                this.throwError(
                    ErrorCode.E001_UNEXPECTED_TOKEN,
                    { token: this.currentToken.type, expected: "表达式" },
                    this.currentToken.loc,
                    [{ message: "检查此处是否有语法错误" }],
                );
        }
    }

    /**
     * 解析标识符节点（仅返回Identifier，不处理调用或成员访问）
     */
    private parseIdentifierNode(): Identifier {
        if (!this.isToken(TokenType.IDENTIFIER)) {
            this.throwError(
                ErrorCode.E006_INVALID_IDENTIFIER,
                { identifier: this.currentToken.value || this.currentToken.type },
                this.currentToken.loc,
                [{ message: "此处需要一个有效的标识符" }],
            );
        }
        const token = this.currentToken;
        this.advance();
        return createIdentifier(token.value, token.loc);
    }

    /**
     * 解析super表达式
     * @returns super表达式节点
     */
    private parseSuperExpression(): SuperExpression {
        const startLoc = this.currentToken.loc;
        this.advance();

        if (this.isToken(TokenType.LEFT_PAREN)) {
            this.advance();
            this.expectAndConsume(TokenType.RIGHT_PAREN, ")");
        }

        return createSuperExpression(this.createLoc(startLoc));
    }

    /**
     * 解析标识符表达式（可能包含调用或成员访问）
     */
    private parseIdentifierExpression(): Expression {
        const token = this.currentToken;
        const identifier = createIdentifier(token.value, token.loc);
        this.advance();

        if (this.isToken(TokenType.LEFT_PAREN)) {
            return this.parseCallExpression(identifier);
        }

        if (this.isToken(TokenType.DOT) || this.isToken(TokenType.LEFT_BRACKET)) {
            return this.parseMemberExpression(identifier);
        }

        return identifier;
    }

    /**
     * 解析数字字面量
     * @returns 字面量节点
     */
    private parseNumberLiteral(): Literal {
        const token = this.currentToken;
        const value = parseFloat(token.value);
        this.advance();
        return createLiteral(value, token.value, token.loc);
    }

    /**
     * 解析字符串字面量
     * @returns 字面量节点
     */
    private parseStringLiteral(): Literal {
        const token = this.currentToken;
        this.advance();
        return createLiteral(token.value, `"${token.value}"`, token.loc);
    }

    /**
     * 解析布尔字面量
     * @returns 字面量节点
     */
    private parseBooleanLiteral(): Literal {
        const token = this.currentToken;
        const value = token.value === "true";
        this.advance();
        return createLiteral(value, token.value, token.loc);
    }

    /**
     * 解析字符字面量
     * @returns 字面量节点
     */
    private parseCharacterLiteral(): Literal {
        const token = this.currentToken;
        this.advance();
        return createLiteral(token.value, `'${token.value}'`, token.loc);
    }

    /**
     * 解析括号表达式或元组表达式
     * @returns 表达式节点
     */
    private parseParenthesizedExpression(): Expression {
        const startLoc = this.currentToken.loc;
        this.advance();

        if (this.isToken(TokenType.RIGHT_PAREN)) {
            this.advance();
            return createTupleExpression([], this.createLoc(startLoc));
        }

        const firstExpr = this.parseExpression();

        if (this.isToken(TokenType.COMMA)) {
            const elements: Expression[] = [firstExpr];
            while (this.isToken(TokenType.COMMA)) {
                this.advance();
                if (this.isToken(TokenType.RIGHT_PAREN)) {
                    break;
                }
                elements.push(this.parseExpression());
            }
            this.expectAndConsume(TokenType.RIGHT_PAREN, ")");
            return createTupleExpression(elements, this.createLoc(startLoc));
        }

        this.expectAndConsume(TokenType.RIGHT_PAREN, ")");
        return firstExpr;
    }

    /**
     * 解析数组表达式
     * @returns 数组表达式节点
     */
    private parseArrayExpression(): ArrayExpression {
        const startLoc = this.currentToken.loc;
        this.advance();

        const elements: Expression[] = [];
        while (!this.isToken(TokenType.RIGHT_BRACKET)) {
            elements.push(this.parseExpression());

            if (this.isToken(TokenType.COMMA)) {
                this.advance();
            } else if (!this.isToken(TokenType.RIGHT_BRACKET)) {
                this.throwError(
                    ErrorCode.E002_EXPECTED_TOKEN,
                    { expected: ", 或 ]", found: this.currentToken.type },
                    this.currentToken.loc,
                );
            }
        }

        this.advance();

        return createArrayExpression(elements, this.createLoc(startLoc));
    }

    /**
     * 解析对象表达式
     * @returns 对象表达式节点
     */
    private parseObjectExpression(): ObjectExpression {
        const startLoc = this.currentToken.loc;
        this.advance();

        const properties: ObjectProperty[] = [];
        while (!this.isToken(TokenType.RIGHT_BRACE)) {
            if (this.isToken(TokenType.IDENTIFIER)) {
                const key = this.parseIdentifierNode();

                if (this.isToken(TokenType.COLON)) {
                    this.advance();
                    const value = this.parseExpression();
                    properties.push(createObjectProperty(key, value, false));
                } else if (this.isToken(TokenType.COMMA) || this.isToken(TokenType.RIGHT_BRACE)) {
                    properties.push(createObjectProperty(key, key, true));
                } else {
                    this.throwError(
                        ErrorCode.E002_EXPECTED_TOKEN,
                        { expected: ": 或 , 或 }", found: this.currentToken.type },
                        this.currentToken.loc,
                    );
                }
            } else if (this.isToken(TokenType.STRING_LITERAL)) {
                const key = createLiteral(
                    this.currentToken.value,
                    `"${this.currentToken.value}"`,
                    this.currentToken.loc,
                );
                this.advance();

                if (this.isToken(TokenType.COLON)) {
                    this.advance();
                    const value = this.parseExpression();
                    properties.push(createObjectProperty(key, value, false));
                } else {
                    this.throwError(
                        ErrorCode.E002_EXPECTED_TOKEN,
                        { expected: ":", found: this.currentToken.type },
                        this.currentToken.loc,
                    );
                }
            } else {
                this.throwError(
                    ErrorCode.E002_EXPECTED_TOKEN,
                    { expected: "标识符 或 字符串", found: this.currentToken.type },
                    this.currentToken.loc,
                );
            }

            if (this.isToken(TokenType.COMMA)) {
                this.advance();
            } else if (!this.isToken(TokenType.RIGHT_BRACE)) {
                this.throwError(
                    ErrorCode.E002_EXPECTED_TOKEN,
                    { expected: ", 或 }", found: this.currentToken.type },
                    this.currentToken.loc,
                );
            }
        }

        this.advance();

        return createObjectExpression(properties, this.createLoc(startLoc));
    }

    /**
     * 解析函数调用表达式
     * @param callee 被调用的函数
     * @returns 函数调用表达式节点
     */
    private parseCallExpression(callee: Expression): Expression {
        const startLoc = callee.loc;
        this.advance();

        const args: Expression[] = [];
        while (!this.isToken(TokenType.RIGHT_PAREN)) {
            args.push(this.parseExpression());

            if (this.isToken(TokenType.COMMA)) {
                this.advance();
            } else if (!this.isToken(TokenType.RIGHT_PAREN)) {
                this.throwError(
                    ErrorCode.E002_EXPECTED_TOKEN,
                    { expected: ", 或 )", found: this.currentToken.type },
                    this.currentToken.loc,
                );
            }
        }

        this.advance();

        const callExpr = createCallExpression(callee, args, {
            start: startLoc?.start || { line: 0, column: 0 },
            end: this.currentToken.loc?.end || { line: 0, column: 0 },
        });

        const expr = this.parsePostfixExpression(callExpr);
        return expr;
    }

    /**
     * 解析成员访问表达式
     * @param object 对象表达式
     * @returns 成员访问表达式节点
     */
    private parseMemberExpression(object: Expression): Expression {
        let memberExpr: Expression = object;

        while (this.isToken(TokenType.DOT) || this.isToken(TokenType.LEFT_BRACKET)) {
            if (this.isToken(TokenType.DOT)) {
                this.advance();

                let property: Expression;
                if (this.isToken(TokenType.NUMBER_LITERAL)) {
                    property = this.parseNumberLiteral();
                } else {
                    property = this.parseIdentifierNode();
                }
                memberExpr = createMemberExpression(memberExpr, property, false);

                if (this.isToken(TokenType.LEFT_PAREN)) {
                    memberExpr = this.parseCallExpression(memberExpr);
                }
            } else {
                this.advance();

                const property = this.parseExpression();
                this.expectAndConsume(TokenType.RIGHT_BRACKET, "]");

                memberExpr = createMemberExpression(memberExpr, property, true);

                if (this.isToken(TokenType.LEFT_PAREN)) {
                    memberExpr = this.parseCallExpression(memberExpr);
                }
            }
        }

        return memberExpr;
    }

    /**
     * 解析后缀表达式（处理成员访问和链式调用）
     */
    private parsePostfixExpression(expr: Expression): Expression {
        if (this.isToken(TokenType.DOT) || this.isToken(TokenType.LEFT_BRACKET)) {
            return this.parseMemberExpression(expr);
        }

        if (this.isToken(TokenType.LEFT_PAREN)) {
            return this.parseCallExpression(expr);
        }

        return expr;
    }

    /**
     * 检查当前token是否为指定类型
     */
    private isToken(type: TokenType): boolean {
        return this.currentToken.type === type;
    }

    /**
     * 期望并消费指定类型的token
     */
    private expectAndConsume(type: TokenType, expected: string): void {
        if (!this.isToken(type)) {
            this.throwError(
                ErrorCode.E002_EXPECTED_TOKEN,
                { expected, found: this.currentToken.type },
                this.currentToken.loc,
            );
        }
        this.advance();
    }

    /**
     * 获取指定行的源代码
     */
    private getSourceLine(line: number): string | undefined {
        const lines = this.source.split("\n");
        if (line > 0 && line <= lines.length) {
            return lines[line - 1];
        }
        return undefined;
    }

    /**
     * 抛出结构化错误
     */
    private throwError(
        code: ErrorCode,
        params: Record<string, string>,
        range?: SourceRange,
        suggestions?: ErrorSuggestion[],
    ): never {
        const sourceLine = range ? this.getSourceLine(range.start.line) : undefined;
        const error = createError(code, params, {
            range,
            filePath: this.filePath,
            sourceLine,
            suggestions,
        });
        throw new ParseError(error);
    }

    /**
     * 消费可选的分号
     */
    private consumeOptionalSemicolon(): void {
        if (this.isToken(TokenType.SEMICOLON)) {
            this.advance();
        }
    }

    /**
     * 创建位置信息
     */
    private createLoc(startLoc: Token["loc"]): Node["loc"] | undefined {
        if (!startLoc) return undefined;
        return {
            start: startLoc.start,
            end: this.currentToken.loc?.end || { line: 0, column: 0 },
        };
    }
}

/**
 * 创建解析器
 * @param source 模板源代码
 * @param language 语言配置
 * @param filePath 文件路径（可选，用于错误报告）
 * @returns 解析器实例
 */
export function createParser(source: string, language?: DejavuLanguage, filePath?: string): Parser {
    return new Parser(source, language, filePath);
}

/**
 * 解析模板，生成AST
 * @param source 模板源代码
 * @param language 语言配置
 * @param filePath 文件路径（可选，用于错误报告）
 * @returns 程序节点
 */
export function parse(source: string, language?: DejavuLanguage, filePath?: string): Program {
    const parser = new Parser(source, language, filePath);
    return parser.parse();
}
