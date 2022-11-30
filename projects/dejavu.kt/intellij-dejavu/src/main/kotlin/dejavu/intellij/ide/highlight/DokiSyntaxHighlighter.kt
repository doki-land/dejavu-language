package dejavu.intellij.ide.highlight

import com.intellij.lexer.Lexer
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.fileTypes.SyntaxHighlighter
import com.intellij.openapi.fileTypes.SyntaxHighlighterBase
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import dejavu.intellij.language.DokiLanguage
import dejavu.intellij.language.parser.DejavuLexer
import dejavu.intellij.language.psi.DejavuTypes

class DokiSyntaxHighlighter : SyntaxHighlighter {
    override fun getHighlightingLexer(): Lexer {
        return DejavuLexer(DokiLanguage.LanguageConfig)
    }

    override fun getTokenHighlights(tokenType: IElementType): Array<out TextAttributesKey?> {
        return SyntaxHighlighterBase.pack(getTokenColor(tokenType)?.textAttributesKey)
    }

    private fun getTokenColor(tokenType: IElementType): HighlightColor? {
        return when (tokenType) {
            // 模板标记
            DejavuTypes.SLOT_L, DejavuTypes.SLOT_R -> HighlightColor.EXTENSION
            DejavuTypes.COMMENT_L, DejavuTypes.COMMENT_R -> HighlightColor.COMMENT_BLOCK
            DejavuTypes.COMMENT_CONTENT -> HighlightColor.COMMENT_BLOCK

            // 关键词
            DejavuTypes.KEYWORD_RAW,
            DejavuTypes.KEYWORD_IF, DejavuTypes.KEYWORD_ELSE,
            DejavuTypes.KEYWORD_LOOP, DejavuTypes.KEYWORD_WHILE, DejavuTypes.KEYWORD_UNTIL,
            DejavuTypes.KEYWORD_MATCH, DejavuTypes.KEYWORD_CASE,
            DejavuTypes.KEYWORD_BLOCK, DejavuTypes.KEYWORD_EXTENDS, DejavuTypes.KEYWORD_INCLUDE,
            DejavuTypes.KEYWORD_END -> HighlightColor.KEYWORD

            // 程序标记
            DejavuTypes.IDENTIFIER -> HighlightColor.IDENTIFIER
            DejavuTypes.NUMBER -> HighlightColor.INTEGER
            DejavuTypes.STRING -> HighlightColor.STRING
            DejavuTypes.PLUS, DejavuTypes.MINUS, DejavuTypes.MULTIPLY, DejavuTypes.DIVIDE, DejavuTypes.MODULO,
            DejavuTypes.EQUAL, DejavuTypes.NOT_EQUAL, DejavuTypes.LESS_THAN, DejavuTypes.GREATER_THAN,
            DejavuTypes.LESS_THAN_OR_EQUAL, DejavuTypes.GREATER_THAN_OR_EQUAL, DejavuTypes.AND, DejavuTypes.OR,
            DejavuTypes.NOT, DejavuTypes.ASSIGN, DejavuTypes.PLUS_ASSIGN, DejavuTypes.MINUS_ASSIGN,
            DejavuTypes.MULTIPLY_ASSIGN, DejavuTypes.DIVIDE_ASSIGN, DejavuTypes.MODULO_ASSIGN -> HighlightColor.OPERATOR

            DejavuTypes.PUNCTUATION -> HighlightColor.PARENTHESES

            // 文本
            DejavuTypes.TEXT -> HighlightColor.IDENTIFIER

            // 错误
            TokenType.BAD_CHARACTER -> HighlightColor.BAD_CHARACTER

            else -> null
        }
    }
}
