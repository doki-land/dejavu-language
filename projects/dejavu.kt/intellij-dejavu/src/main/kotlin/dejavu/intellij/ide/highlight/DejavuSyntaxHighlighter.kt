package dejavu.intellij.ide.highlight

import com.intellij.lexer.Lexer
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.fileTypes.SyntaxHighlighter
import com.intellij.openapi.fileTypes.SyntaxHighlighterBase.pack
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import dejavu.intellij.language.DejavuLanguage
import dejavu.intellij.language.parser.DejavuLexer
import dejavu.intellij.language.psi.DejavuTypes
import dejavu.intellij.ide.highlight.HighlightColor as Color

class DejavuSyntaxHighlighter : SyntaxHighlighter {
    override fun getHighlightingLexer(): Lexer {
        return DejavuLexer(DejavuLanguage.LanguageConfig)
    }

    override fun getTokenHighlights(tokenType: IElementType): Array<TextAttributesKey> {
        return pack(getTokenColor(tokenType)?.textAttributesKey)
    }

    private fun getTokenColor(tokenType: IElementType): Color? {
        return when (tokenType) {
            // 模板标记
            DejavuTypes.SLOT_L, DejavuTypes.SLOT_R -> Color.DELIMITER
            DejavuTypes.COMMENT_L, DejavuTypes.COMMENT_R -> Color.COMMENT_BLOCK
            DejavuTypes.COMMENT_CONTENT -> Color.COMMENT_BLOCK

            // 关键词
            DejavuTypes.KEYWORD_IF, DejavuTypes.KEYWORD_END, DejavuTypes.KEYWORD_LOOP, DejavuTypes.KEYWORD_MATCH,
            DejavuTypes.KEYWORD_CASE, DejavuTypes.KEYWORD_ELSE, DejavuTypes.KEYWORD_WHILE, DejavuTypes.KEYWORD_UNTIL,
            DejavuTypes.KEYWORD_EXTENDS, DejavuTypes.KEYWORD_BLOCK, DejavuTypes.KEYWORD_INCLUDE, DejavuTypes.KEYWORD_RAW,
            DejavuTypes.KEYWORD_LET, DejavuTypes.KEYWORD_IN -> Color.KEYWORD

            // Macro 关键词
            DejavuTypes.KEYWORD_MACRO -> Color.SYM_MACRO

            // 程序标记
            DejavuTypes.IDENTIFIER -> Color.IDENTIFIER
            DejavuTypes.NUMBER -> Color.INTEGER
            DejavuTypes.STRING -> Color.STRING
            DejavuTypes.PLUS, DejavuTypes.MINUS, DejavuTypes.MULTIPLY, DejavuTypes.DIVIDE, DejavuTypes.MODULO,
            DejavuTypes.EQUAL, DejavuTypes.NOT_EQUAL, DejavuTypes.LESS_THAN, DejavuTypes.GREATER_THAN,
            DejavuTypes.LESS_THAN_OR_EQUAL, DejavuTypes.GREATER_THAN_OR_EQUAL, DejavuTypes.AND, DejavuTypes.OR,
            DejavuTypes.NOT, DejavuTypes.ASSIGN, DejavuTypes.PLUS_ASSIGN, DejavuTypes.MINUS_ASSIGN,
            DejavuTypes.MULTIPLY_ASSIGN, DejavuTypes.DIVIDE_ASSIGN, DejavuTypes.MODULO_ASSIGN -> Color.OPERATOR

            DejavuTypes.PUNCTUATION -> Color.PARENTHESES

            // 文本
//            DejavuTypes.TEXT -> Color.IDENTIFIER

            // 错误
            TokenType.BAD_CHARACTER -> Color.BAD_CHARACTER

            else -> null
        }
    }
}
