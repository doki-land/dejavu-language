package dejavu.intellij.language.parser.templates

import com.intellij.lang.PsiBuilder
import dejavu.intellij.language.parser.fragments.FragmentParsers
import dejavu.intellij.language.psi.DejavuTypes

/**
 * 模板解析器
 * 使用组合模式，不继承任何基类
 */
object TemplateParsers {

    /**
     * 解析 if 模板
     * ```dejavu
     * <% if <EXPRESSION> %>
     *     ...
     * <% else if <EXPRESSION> %>
     *     ...
     * <% else %>
     *     ...
     * <% end %>
     * ```
     */
    fun parseIfTemplate(builder: PsiBuilder) {
        val templateMarker = builder.mark()

        // 解析 if fragment
        FragmentParsers.parseIfFragment(builder)

        // 解析 if 块内容
        parseIfBlock(builder)

        // 解析 else if / else / end
        while (!builder.eof() && builder.tokenType == DejavuTypes.SLOT_L) {
            val keyword = FragmentParsers.detectKeyword(builder)

            when (keyword) {
                DejavuTypes.KEYWORD_ELSE -> {
                    FragmentParsers.parseElseFragment(builder)
                    parseElseBlock(builder)
                }

                DejavuTypes.KEYWORD_END -> {
                    FragmentParsers.parseEndFragment(builder)
                    break
                }

                else -> break
            }
        }

        templateMarker.done(DejavuTypes.IF_TEMPLATE)
    }

    /**
     * 解析 if 块内容
     */
    private fun parseIfBlock(builder: PsiBuilder) {
        while (!builder.eof()) {
            when (builder.tokenType) {
                DejavuTypes.SLOT_L -> {
                    val keyword = FragmentParsers.detectKeyword(builder)
                    when {
                        keyword == DejavuTypes.KEYWORD_END || keyword == DejavuTypes.KEYWORD_ELSE -> return
                        keyword == DejavuTypes.KEYWORD_IF -> parseIfTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_LOOP -> parseLoopTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_MATCH -> parseMatchTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_WHILE -> parseWhileTemplate(builder)
                        else -> FragmentParsers.parseInterpolationFragment(builder)
                    }
                }

                DejavuTypes.COMMENT_L -> FragmentParsers.parseComment(builder)
                DejavuTypes.TEXT -> FragmentParsers.parseText(builder)
                else -> if (!builder.eof()) builder.advanceLexer()
            }
        }
    }

    /**
     * 解析 else 块内容
     */
    private fun parseElseBlock(builder: PsiBuilder) {
        while (!builder.eof()) {
            when (builder.tokenType) {
                DejavuTypes.SLOT_L -> {
                    val keyword = FragmentParsers.detectKeyword(builder)
                    when {
                        keyword == DejavuTypes.KEYWORD_END -> return
                        keyword == DejavuTypes.KEYWORD_IF -> parseIfTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_LOOP -> parseLoopTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_MATCH -> parseMatchTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_WHILE -> parseWhileTemplate(builder)
                        else -> FragmentParsers.parseInterpolationFragment(builder)
                    }
                }

                DejavuTypes.COMMENT_L -> FragmentParsers.parseComment(builder)
                DejavuTypes.TEXT -> FragmentParsers.parseText(builder)
                else -> if (!builder.eof()) builder.advanceLexer()
            }
        }
    }

    /**
     * 解析 loop 模板
     * ```dejavu
     * <% loop <EXPRESSION> %>
     *     ...
     * <% end %>
     * ```
     */
    fun parseLoopTemplate(builder: PsiBuilder) {
        val templateMarker = builder.mark()

        FragmentParsers.parseLoopFragment(builder)
        parseLoopBlock(builder)

        if (!builder.eof() && builder.tokenType == DejavuTypes.SLOT_L) {
            if (FragmentParsers.detectKeyword(builder) == DejavuTypes.KEYWORD_END) {
                FragmentParsers.parseEndFragment(builder)
            }
        }

        templateMarker.done(DejavuTypes.LOOP_TEMPLATE)
    }

    /**
     * 解析 loop 块内容
     */
    private fun parseLoopBlock(builder: PsiBuilder) {
        while (!builder.eof()) {
            when (builder.tokenType) {
                DejavuTypes.SLOT_L -> {
                    val keyword = FragmentParsers.detectKeyword(builder)
                    when {
                        keyword == DejavuTypes.KEYWORD_END -> return
                        keyword == DejavuTypes.KEYWORD_IF -> parseIfTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_LOOP -> parseLoopTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_MATCH -> parseMatchTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_WHILE -> parseWhileTemplate(builder)
                        else -> FragmentParsers.parseInterpolationFragment(builder)
                    }
                }

                DejavuTypes.COMMENT_L -> FragmentParsers.parseComment(builder)
                DejavuTypes.TEXT -> FragmentParsers.parseText(builder)
                else -> if (!builder.eof()) builder.advanceLexer()
            }
        }
    }

    /**
     * 解析 while 模板
     * ```dejavu
     * <% while <EXPRESSION> %>
     *     ...
     * <% end %>
     * ```
     */
    fun parseWhileTemplate(builder: PsiBuilder) {
        val templateMarker = builder.mark()

        FragmentParsers.parseWhileFragment(builder)
        parseWhileBlock(builder)

        if (!builder.eof() && builder.tokenType == DejavuTypes.SLOT_L) {
            if (FragmentParsers.detectKeyword(builder) == DejavuTypes.KEYWORD_END) {
                FragmentParsers.parseEndFragment(builder)
            }
        }

        templateMarker.done(DejavuTypes.WHILE_TEMPLATE)
    }

    /**
     * 解析 while 块内容
     */
    private fun parseWhileBlock(builder: PsiBuilder) {
        while (!builder.eof()) {
            when (builder.tokenType) {
                DejavuTypes.SLOT_L -> {
                    val keyword = FragmentParsers.detectKeyword(builder)
                    when {
                        keyword == DejavuTypes.KEYWORD_END -> return
                        keyword == DejavuTypes.KEYWORD_IF -> parseIfTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_LOOP -> parseLoopTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_MATCH -> parseMatchTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_WHILE -> parseWhileTemplate(builder)
                        else -> FragmentParsers.parseInterpolationFragment(builder)
                    }
                }

                DejavuTypes.COMMENT_L -> FragmentParsers.parseComment(builder)
                DejavuTypes.TEXT -> FragmentParsers.parseText(builder)
                else -> if (!builder.eof()) builder.advanceLexer()
            }
        }
    }

    /**
     * 解析 match 模板
     * ```dejavu
     * <% match <EXPRESSION> %>
     *     <% case <EXPRESSION> %>
     *         ...
     * <% end %>
     * ```
     */
    fun parseMatchTemplate(builder: PsiBuilder) {
        val templateMarker = builder.mark()

        FragmentParsers.parseMatchFragment(builder)
        parseMatchBlock(builder)

        if (!builder.eof() && builder.tokenType == DejavuTypes.SLOT_L) {
            if (FragmentParsers.detectKeyword(builder) == DejavuTypes.KEYWORD_END) {
                FragmentParsers.parseEndFragment(builder)
            }
        }

        templateMarker.done(DejavuTypes.MATCH_TEMPLATE)
    }

    /**
     * 解析 match 块内容
     */
    private fun parseMatchBlock(builder: PsiBuilder) {
        while (!builder.eof()) {
            when (builder.tokenType) {
                DejavuTypes.SLOT_L -> {
                    val keyword = FragmentParsers.detectKeyword(builder)
                    when {
                        keyword == DejavuTypes.KEYWORD_END -> return
                        keyword == DejavuTypes.KEYWORD_IF -> parseIfTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_LOOP -> parseLoopTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_MATCH -> parseMatchTemplate(builder)
                        keyword == DejavuTypes.KEYWORD_WHILE -> parseWhileTemplate(builder)
                        else -> FragmentParsers.parseInterpolationFragment(builder)
                    }
                }

                DejavuTypes.COMMENT_L -> FragmentParsers.parseComment(builder)
                DejavuTypes.TEXT -> FragmentParsers.parseText(builder)
                else -> if (!builder.eof()) builder.advanceLexer()
            }
        }
    }

    /**
     * 解析 block 模板
     * ```dejavu
     * <% block <IDENTIFIER> %>
     *     ...
     * <% end %>
     * ```
     */
    fun parseBlockTemplate(builder: PsiBuilder) {
        val templateMarker = builder.mark()

        FragmentParsers.parseBlockFragment(builder)
        parseBlockContent(builder)

        if (!builder.eof() && builder.tokenType == DejavuTypes.SLOT_L) {
            if (FragmentParsers.detectKeyword(builder) == DejavuTypes.KEYWORD_END) {
                FragmentParsers.parseEndFragment(builder)
            }
        }

        templateMarker.done(DejavuTypes.BLOCK_TEMPLATE)
    }

    /**
     * 解析 block 内容
     */
    private fun parseBlockContent(builder: PsiBuilder) {
        while (!builder.eof()) {
            when (builder.tokenType) {
                DejavuTypes.SLOT_L -> {
                    if (FragmentParsers.detectKeyword(builder) == DejavuTypes.KEYWORD_END) {
                        return
                    }
                    FragmentParsers.parseInterpolationFragment(builder)
                }

                DejavuTypes.COMMENT_L -> FragmentParsers.parseComment(builder)
                DejavuTypes.TEXT -> FragmentParsers.parseText(builder)
                else -> if (!builder.eof()) builder.advanceLexer()
            }
        }
    }

    /**
     * 解析 raw 模板
     * ```dejavu
     * <% raw %>
     *     ...
     * <% end raw %>
     * ```
     */
    fun parseRawTemplate(builder: PsiBuilder) {
        val templateMarker = builder.mark()

        FragmentParsers.parseRawFragment(builder)
        parseRawContent(builder)

        if (!builder.eof() && builder.tokenType == DejavuTypes.SLOT_L) {
            FragmentParsers.parseEndFragment(builder)
        }

        templateMarker.done(DejavuTypes.RAW_TEMPLATE)
    }

    /**
     * 解析 raw 内容（不解析模板语法）
     */
    private fun parseRawContent(builder: PsiBuilder) {
        while (!builder.eof()) {
            when (builder.tokenType) {
                DejavuTypes.SLOT_L -> {
                    val marker = builder.mark()
                    builder.advanceLexer()
                    FragmentParsers.skipWhitespace(builder)

                    if (builder.tokenType == DejavuTypes.KEYWORD_END) {
                        builder.advanceLexer()
                        FragmentParsers.skipWhitespace(builder)

                        if (builder.tokenType == DejavuTypes.KEYWORD_RAW) {
                            marker.rollbackTo()
                            return
                        }
                    }
                    marker.drop()
                }

                else -> if (!builder.eof()) builder.advanceLexer()
            }
        }
    }

    /**
     * 解析 macro 模板
     * ```dejavu
     * <% macro <IDENTIFIER>(<PARAMS>) %>
     *     ...
     * <% end macro %>
     * ```
     */
    fun parseMacroTemplate(builder: PsiBuilder) {
        val templateMarker = builder.mark()

        FragmentParsers.parseMacroFragment(builder)
        parseMacroBlock(builder)

        if (!builder.eof() && builder.tokenType == DejavuTypes.SLOT_L) {
            FragmentParsers.parseEndFragment(builder)
        }

        templateMarker.done(DejavuTypes.MACRO_TEMPLATE)
    }

    /**
     * 解析 macro 块内容
     */
    private fun parseMacroBlock(builder: PsiBuilder) {
        while (!builder.eof()) {
            when (builder.tokenType) {
                DejavuTypes.SLOT_L -> {
                    val keyword = FragmentParsers.detectKeyword(builder)
                    if (keyword == DejavuTypes.KEYWORD_END) {
                        return
                    }
                    FragmentParsers.parseInterpolationFragment(builder)
                }

                DejavuTypes.COMMENT_L -> FragmentParsers.parseComment(builder)
                DejavuTypes.TEXT -> FragmentParsers.parseText(builder)
                else -> if (!builder.eof()) builder.advanceLexer()
            }
        }
    }
}
