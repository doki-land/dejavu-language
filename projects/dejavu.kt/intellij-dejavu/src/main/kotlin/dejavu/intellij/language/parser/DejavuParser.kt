package dejavu.intellij.language.parser

import com.intellij.lang.ASTNode
import com.intellij.lang.PsiBuilder
import com.intellij.lang.PsiParser
import com.intellij.psi.tree.IElementType
import dejavu.intellij.language.DejavuLanguage
import dejavu.intellij.language.TemplateConfig
import dejavu.intellij.language.parser.fragments.FragmentParsers
import dejavu.intellij.language.parser.templates.TemplateParsers
import dejavu.intellij.language.psi.DejavuTypes

/**
 * Dejavu 模板语言解析器
 * 使用组合模式，不继承 BaseParser
 */
class DejavuParser(private val config: TemplateConfig = DejavuLanguage.LanguageConfig) : PsiParser {

    override fun parse(root: IElementType, builder: PsiBuilder): ASTNode {
        val rootMarker = builder.mark()

        while (!builder.eof()) {
            when (builder.tokenType) {
                DejavuTypes.SLOT_L -> {
                    val keyword = FragmentParsers.detectKeyword(builder)
                    when (keyword) {
                        DejavuTypes.KEYWORD_IF -> TemplateParsers.parseIfTemplate(builder)
                        DejavuTypes.KEYWORD_LOOP -> TemplateParsers.parseLoopTemplate(builder)
                        DejavuTypes.KEYWORD_MATCH -> TemplateParsers.parseMatchTemplate(builder)
                        DejavuTypes.KEYWORD_WHILE -> TemplateParsers.parseWhileTemplate(builder)
                        DejavuTypes.KEYWORD_RAW -> TemplateParsers.parseRawTemplate(builder)
                        DejavuTypes.KEYWORD_BLOCK -> TemplateParsers.parseBlockTemplate(builder)
                        DejavuTypes.KEYWORD_MACRO -> TemplateParsers.parseMacroTemplate(builder)
                        else -> parseGenericFragment(builder)
                    }
                }

                DejavuTypes.COMMENT_L -> FragmentParsers.parseComment(builder)
                DejavuTypes.TEXT -> FragmentParsers.parseText(builder)
                else -> builder.advanceLexer()
            }
        }

        rootMarker.done(root)
        return builder.treeBuilt
    }

    /**
     * 解析通用 fragment
     */
    private fun parseGenericFragment(builder: PsiBuilder) {
        val marker = builder.mark()
        builder.advanceLexer()
        FragmentParsers.skipWhitespace(builder)

        when (builder.tokenType) {
            DejavuTypes.KEYWORD_CASE -> {
                marker.rollbackTo()
                FragmentParsers.parseCaseFragment(builder)
            }

            DejavuTypes.KEYWORD_ELSE -> {
                marker.rollbackTo()
                FragmentParsers.parseElseFragment(builder)
            }

            DejavuTypes.KEYWORD_END -> {
                marker.rollbackTo()
                FragmentParsers.parseEndFragment(builder)
            }

            DejavuTypes.KEYWORD_UNTIL -> {
                marker.rollbackTo()
                FragmentParsers.parseUntilFragment(builder)
            }

            DejavuTypes.KEYWORD_EXTENDS -> {
                marker.rollbackTo()
                FragmentParsers.parseExtendsFragment(builder)
            }

            DejavuTypes.KEYWORD_INCLUDE -> {
                marker.rollbackTo()
                FragmentParsers.parseIncludeFragment(builder)
            }

            null -> marker.done(DejavuTypes.EXPRESSION_TEMPLATE)
            else -> {
                FragmentParsers.parseExpression(builder)
                FragmentParsers.consumeClosingDelimiter(builder)
                marker.done(DejavuTypes.EXPRESSION_TEMPLATE)
            }
        }
    }
}
