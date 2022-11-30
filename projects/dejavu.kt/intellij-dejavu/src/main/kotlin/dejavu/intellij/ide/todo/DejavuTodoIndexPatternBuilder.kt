package dejavu.intellij.ide.todo

import com.intellij.lexer.Lexer
import com.intellij.psi.PsiFile
import com.intellij.psi.impl.search.IndexPatternBuilder
import com.intellij.psi.tree.IElementType
import com.intellij.psi.tree.TokenSet
import dejavu.intellij.language.TemplateConfig
import dejavu.intellij.language.file.DejavuFileNode
import dejavu.intellij.language.parser.DejavuLexer
import dejavu.intellij.language.psi.DejavuTypes

class DejavuTodoIndexPatternBuilder : IndexPatternBuilder {
    override fun getIndexingLexer(file: PsiFile): Lexer? =
        if (file is DejavuFileNode) DejavuLexer(TemplateConfig()) else null

    override fun getCommentTokenSet(file: PsiFile): TokenSet? =
        if (file is DejavuFileNode) TokenSet.create(DejavuTypes.COMMENT_CONTENT) else null

    override fun getCommentStartDelta(tokenType: IElementType?): Int =
        if (tokenType == DejavuTypes.COMMENT_CONTENT) 2 else 0

    override fun getCommentEndDelta(tokenType: IElementType?): Int = 0
}
