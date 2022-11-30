package dejavu.intellij.language.parser.fragments

import com.intellij.lang.PsiBuilder
import com.intellij.psi.tree.IElementType
import dejavu.intellij.language.psi.DejavuTypes

/**
 * Fragment 解析工具类
 * 提供通用的 fragment 解析功能，通过组合使用
 *
 * Fragment 是模板中的基本单元，格式为 `<% ... %>`，包括：
 * - 控制流 fragment：if, else, else if, end
 * - 循环 fragment：loop, while, until
 * - 匹配 fragment：match, case
 * - 模板 fragment：extends, include, block
 * - 原始 fragment：raw
 * - 插值 fragment：其他任意非关键词 IDENTIFIER
 */
object FragmentParsers {
    /**
     * 跳过空白 token
     */
    fun skipWhitespace(builder: PsiBuilder) {
        while (!builder.eof() && builder.tokenType == DejavuTypes.WHITESPACE) {
            builder.advanceLexer()
        }
    }

    /**
     * 解析表达式（简化版，消费到结束定界符）
     */
    fun parseExpression(builder: PsiBuilder) {
        while (!builder.eof() && builder.tokenType != DejavuTypes.SLOT_R) {
            builder.advanceLexer()
        }
    }

    /**
     * 检测下一个非空白 token 的关键词类型
     *
     * @param builder PsiBuilder
     * @return 关键词类型，如果不是 SLOT_L 开头则返回 null
     */
    fun detectKeyword(builder: PsiBuilder): IElementType? {
        if (builder.tokenType != DejavuTypes.SLOT_L) {
            return null
        }

        val marker = builder.mark()
        builder.advanceLexer()
        skipWhitespace(builder)
        val keyword = if (!builder.eof()) builder.tokenType else null
        marker.rollbackTo()
        return keyword
    }

    /**
     * 解析 if fragment
     *
     * ```dejavu
     * <% if <EXPRESSION> %>
     * ```
     */
    fun parseIfFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // if
        parseExpression(builder)
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.IF_FRAGMENT)
    }

    /**
     * 解析 else fragment
     *
     * ```dejavu
     * <% else %>
     * <% else if <EXPRESSION> %>
     * ```
     */
    fun parseElseFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // else
        skipWhitespace(builder)

        if (builder.tokenType == DejavuTypes.KEYWORD_IF) {
            builder.advanceLexer() // if
            parseExpression(builder)
            consumeClosingDelimiter(builder)
            marker.done(DejavuTypes.ELSE_IF_FRAGMENT)
        } else {
            consumeClosingDelimiter(builder)
            marker.done(DejavuTypes.ELSE_FRAGMENT)
        }
    }

    /**
     * 解析 end fragment
     *
     * ```dejavu
     * <% end %>
     * <% end raw %>
     * ```
     */
    fun parseEndFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // end
        skipWhitespace(builder)
        while (!builder.eof() && builder.tokenType != DejavuTypes.SLOT_R) {
            builder.advanceLexer()
        }
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.END_FRAGMENT)
    }

    /**
     * 解析 loop fragment
     *
     * ```dejavu
     * <% loop <PATTERN> in <EXPRESSION> %>
     * ```
     */
    fun parseLoopFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // loop
        parseExpression(builder)
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.LOOP_FRAGMENT)
    }

    /**
     * 解析 while fragment
     *
     * ```dejavu
     * <% while <EXPRESSION> %>
     * ```
     */
    fun parseWhileFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // while
        parseExpression(builder)
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.WHILE_FRAGMENT)
    }

    /**
     * 解析 match fragment
     *
     * ```dejavu
     * <% match <EXPRESSION> %>
     * ```
     */
    fun parseMatchFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // match
        parseExpression(builder)
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.MATCH_FRAGMENT)
    }

    /**
     * 解析 case fragment
     *
     * ```dejavu
     * <% case value %>
     * ```
     */
    fun parseCaseFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // case
        parseExpression(builder)
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.CASE_FRAGMENT)
    }

    /**
     * 解析 block fragment
     *
     * ```dejavu
     * <% block name %>
     * ```
     */
    fun parseBlockFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // block
        skipWhitespace(builder)
        if (!builder.eof() && builder.tokenType == DejavuTypes.IDENTIFIER) {
            builder.advanceLexer()
        }
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.BLOCK_FRAGMENT)
    }

    /**
     * 解析 extends fragment
     *
     * ```dejavu
     * <% extends <STRING_LITERAL> %>
     * ```
     */
    fun parseExtendsFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // extends
        parseExpression(builder)
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.EXTENDS_FRAGMENT)
    }

    /**
     * 解析 include fragment
     *
     * ```dejavu
     * <% include <STRING_LITERAL> %>
     * ```
     */
    fun parseIncludeFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // include
        parseExpression(builder)
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.INCLUDE_FRAGMENT)
    }

    /**
     * 解析 raw fragment
     *
     * ```dejavu
     * <% raw %>
     * ```
     */
    fun parseRawFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // raw
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.RAW_FRAGMENT)
    }

    /**
     * 解析 until fragment
     *
     * ```dejavu
     * <% until <EXPRESSION> %>
     * ```
     */
    fun parseUntilFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // until
        parseExpression(builder)
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.UNTIL_FRAGMENT)
    }

    /**
     * 解析 macro fragment
     *
     * ```dejavu
     * <% macro <IDENTIFIER>(<PARAMS>) %>
     * ```
     */
    fun parseMacroFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        builder.advanceLexer() // macro
        skipWhitespace(builder)
        // 解析 macro 名称
        if (!builder.eof() && builder.tokenType == DejavuTypes.IDENTIFIER) {
            builder.advanceLexer()
        }
        // 解析参数列表
        parseExpression(builder)
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.MACRO_FRAGMENT)
    }

    /**
     * 解析插值 fragment
     *
     * ```dejavu
     * <%  expression  %>
     * <%= expression =%>
     * ```
     */
    fun parseInterpolationFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer() // SLOT_L
        skipWhitespace(builder)
        parseExpression(builder)
        consumeClosingDelimiter(builder)
        marker.done(DejavuTypes.EXPRESSION_TEMPLATE)
    }

    /**
     * 解析注释
     *
     * ```dejavu
     * <# comment #>
     * ```
     */
    fun parseComment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer()
        while (!builder.eof() && builder.tokenType != DejavuTypes.COMMENT_R) {
            builder.advanceLexer()
        }
        if (builder.tokenType == DejavuTypes.COMMENT_R) {
            builder.advanceLexer()
        }
        marker.done(DejavuTypes.COMMENT)
    }

    /**
     * 解析文本
     */
    fun parseText(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer()
        marker.done(DejavuTypes.TEXT)
    }

    /**
     * 消费结束定界符 `%>`
     *
     * @param builder PsiBuilder
     * @return 如果成功消费返回 true
     */
    fun consumeClosingDelimiter(builder: PsiBuilder): Boolean {
        return if (builder.tokenType == DejavuTypes.SLOT_R) {
            builder.advanceLexer()
            true
        } else {
            false
        }
    }
}