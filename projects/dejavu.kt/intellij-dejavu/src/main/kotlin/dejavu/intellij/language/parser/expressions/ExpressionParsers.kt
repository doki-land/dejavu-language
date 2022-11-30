package dejavu.intellij.language.parser.expressions

import com.intellij.lang.PsiBuilder
import dejavu.intellij.language.psi.DejavuTypes

/**
 * 表达式解析器
 * 负责解析各种类型的表达式
 * 使用组合模式，不继承任何基类
 *
 * 支持的表达式类型：
 * - 赋值表达式：a = b, a += b
 * - 逻辑表达式：a && b, a || b
 * - 比较表达式：a == b, a < b
 * - 算术表达式：a + b, a * b
 * - 一元表达式：-a, !b
 * - 基本表达式：标识符、数字、字符串、括号表达式
 * - 后缀表达式：函数调用、数组访问、成员访问
 */
object ExpressionParsers {

    /**
     * 解析表达式
     *
     * ```dejavu
     * <%= user.name %>
     * <% items[0] %>
     * <% calculate(a, b) %>
     * ```
     */
    fun parseExpression(builder: PsiBuilder) {
        val exprMarker = builder.mark()
        parseAssignmentExpression(builder)
        exprMarker.done(DejavuTypes.EXPRESSION)
    }

    /**
     * 跳过空白 token
     */
    private fun skipWhitespace(builder: PsiBuilder) {
        while (!builder.eof() && builder.tokenType == DejavuTypes.WHITESPACE) {
            builder.advanceLexer()
        }
    }

    /**
     * 解析赋值表达式
     *
     * ```dejavu
     * x = 10
     * x += 5
     * ```
     */
    private fun parseAssignmentExpression(builder: PsiBuilder) {
        parseLogicalOrExpression(builder)

        skipWhitespace(builder)
        if (!builder.eof() && builder.tokenType in setOf(
                DejavuTypes.ASSIGN,
                DejavuTypes.PLUS_ASSIGN,
                DejavuTypes.MINUS_ASSIGN,
                DejavuTypes.MULTIPLY_ASSIGN,
                DejavuTypes.DIVIDE_ASSIGN,
                DejavuTypes.MODULO_ASSIGN
            )
        ) {
            val exprMarker = builder.mark()
            builder.advanceLexer()
            parseAssignmentExpression(builder)
            exprMarker.done(DejavuTypes.BINARY_EXPRESSION)
        }
    }

    /**
     * 解析逻辑或表达式
     *
     * ```dejavu
     * a || b
     * ```
     */
    private fun parseLogicalOrExpression(builder: PsiBuilder) {
        parseLogicalAndExpression(builder)

        skipWhitespace(builder)
        while (!builder.eof() && builder.tokenType == DejavuTypes.OR) {
            val exprMarker = builder.mark()
            builder.advanceLexer()
            parseLogicalAndExpression(builder)
            exprMarker.done(DejavuTypes.BINARY_EXPRESSION)
        }
    }

    /**
     * 解析逻辑与表达式
     *
     * ```dejavu
     * a && b
     * ```
     */
    private fun parseLogicalAndExpression(builder: PsiBuilder) {
        parseComparisonExpression(builder)

        skipWhitespace(builder)
        while (!builder.eof() && builder.tokenType == DejavuTypes.AND) {
            val exprMarker = builder.mark()
            builder.advanceLexer()
            parseComparisonExpression(builder)
            exprMarker.done(DejavuTypes.BINARY_EXPRESSION)
        }
    }

    /**
     * 解析比较表达式
     *
     * ```dejavu
     * a == b
     * a != b
     * a < b
     * ```
     */
    private fun parseComparisonExpression(builder: PsiBuilder) {
        parseAdditiveExpression(builder)

        skipWhitespace(builder)
        while (!builder.eof() && builder.tokenType in setOf(
                DejavuTypes.EQUAL,
                DejavuTypes.NOT_EQUAL,
                DejavuTypes.LESS_THAN,
                DejavuTypes.GREATER_THAN,
                DejavuTypes.LESS_THAN_OR_EQUAL,
                DejavuTypes.GREATER_THAN_OR_EQUAL
            )
        ) {
            val exprMarker = builder.mark()
            builder.advanceLexer()
            parseAdditiveExpression(builder)
            exprMarker.done(DejavuTypes.BINARY_EXPRESSION)
        }
    }

    /**
     * 解析加法表达式
     *
     * ```dejavu
     * a + b
     * a - b
     * ```
     */
    private fun parseAdditiveExpression(builder: PsiBuilder) {
        parseMultiplicativeExpression(builder)

        skipWhitespace(builder)
        while (!builder.eof() && builder.tokenType in setOf(
                DejavuTypes.PLUS,
                DejavuTypes.MINUS
            )
        ) {
            val exprMarker = builder.mark()
            builder.advanceLexer()
            parseMultiplicativeExpression(builder)
            exprMarker.done(DejavuTypes.BINARY_EXPRESSION)
        }
    }

    /**
     * 解析乘法表达式
     *
     * ```dejavu
     * a * b
     * a / b
     * ```
     */
    private fun parseMultiplicativeExpression(builder: PsiBuilder) {
        parseUnaryExpression(builder)

        skipWhitespace(builder)
        while (!builder.eof() && builder.tokenType in setOf(
                DejavuTypes.MULTIPLY,
                DejavuTypes.DIVIDE,
                DejavuTypes.MODULO
            )
        ) {
            val exprMarker = builder.mark()
            builder.advanceLexer()
            parseUnaryExpression(builder)
            exprMarker.done(DejavuTypes.BINARY_EXPRESSION)
        }
    }

    /**
     * 解析一元表达式
     *
     * ```dejavu
     * -a
     * !condition
     * ```
     */
    private fun parseUnaryExpression(builder: PsiBuilder) {
        skipWhitespace(builder)

        if (!builder.eof() && builder.tokenType in setOf(
                DejavuTypes.PLUS,
                DejavuTypes.MINUS,
                DejavuTypes.NOT
            )
        ) {
            val exprMarker = builder.mark()
            builder.advanceLexer()
            parseUnaryExpression(builder)
            exprMarker.done(DejavuTypes.UNARY_EXPRESSION)
        } else {
            parsePrimaryExpression(builder)
        }
    }

    /**
     * 解析基本表达式
     *
     * ```dejavu
     * identifier
     * 123
     * "string"
     * (expression)
     * ```
     */
    private fun parsePrimaryExpression(builder: PsiBuilder) {
        skipWhitespace(builder)

        // 解析标识符、数字或字符串
        if (!builder.eof() && builder.tokenType in setOf(
                DejavuTypes.IDENTIFIER,
                DejavuTypes.NUMBER,
                DejavuTypes.STRING
            )
        ) {
            val exprMarker = builder.mark()
            builder.advanceLexer()
            parsePostfixExpression(builder)
            exprMarker.done(DejavuTypes.PRIMARY_EXPRESSION)
        }
        // 解析括号表达式
        else if (!builder.eof() && builder.tokenType == DejavuTypes.LPAREN) {
            val exprMarker = builder.mark()
            builder.advanceLexer()
            parseExpression(builder)
            skipWhitespace(builder)
            if (!builder.eof() && builder.tokenType == DejavuTypes.RPAREN) {
                builder.advanceLexer()
            }
            parsePostfixExpression(builder)
            exprMarker.done(DejavuTypes.PRIMARY_EXPRESSION)
        }
    }

    /**
     * 解析后缀表达式
     *
     * ```dejavu
     * func()           // 函数调用
     * array[index]     // 数组访问
     * obj.property     // 成员访问
     * ```
     */
    private fun parsePostfixExpression(builder: PsiBuilder) {
        while (!builder.eof()) {
            skipWhitespace(builder)

            // 处理函数调用 f()
            if (builder.tokenType == DejavuTypes.LPAREN) {
                val callMarker = builder.mark()
                builder.advanceLexer()
                parseArgumentList(builder)
                if (!builder.eof() && builder.tokenType == DejavuTypes.RPAREN) {
                    builder.advanceLexer()
                }
                callMarker.done(DejavuTypes.FUNCTION_CALL)
            }
            // 处理数组访问 f[]
            else if (builder.tokenType == DejavuTypes.LBRACKET) {
                val accessMarker = builder.mark()
                builder.advanceLexer()
                parseExpression(builder)
                if (!builder.eof() && builder.tokenType == DejavuTypes.RBRACKET) {
                    builder.advanceLexer()
                }
                accessMarker.done(DejavuTypes.ARRAY_ACCESS)
            }
            // 处理成员访问 f.
            else if (builder.tokenType == DejavuTypes.DOT) {
                val memberMarker = builder.mark()
                builder.advanceLexer()
                skipWhitespace(builder)
                if (!builder.eof() && builder.tokenType == DejavuTypes.IDENTIFIER) {
                    builder.advanceLexer()
                }
                memberMarker.done(DejavuTypes.MEMBER_ACCESS)
            } else {
                break
            }
        }
    }

    /**
     * 解析参数列表
     *
     * ```dejavu
     * func(a, b, c)
     * ```
     */
    private fun parseArgumentList(builder: PsiBuilder) {
        skipWhitespace(builder)
        while (!builder.eof() && builder.tokenType != DejavuTypes.RPAREN) {
            parseExpression(builder)
            skipWhitespace(builder)
            if (builder.tokenType == DejavuTypes.RPAREN) {
                break
            }
            if (builder.tokenType == DejavuTypes.COMMA) {
                builder.advanceLexer()
                skipWhitespace(builder)
            } else {
                break
            }
        }
    }
}
