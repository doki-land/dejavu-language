package dejavu.intellij.ide.todo

import com.intellij.lexer.Lexer
import com.intellij.psi.impl.cache.impl.BaseFilterLexer
import com.intellij.psi.impl.cache.impl.OccurrenceConsumer
import com.intellij.psi.impl.cache.impl.todo.LexerBasedTodoIndexer
import com.intellij.psi.search.UsageSearchContext
import dejavu.intellij.language.TemplateConfig
import dejavu.intellij.language.parser.DejavuLexer
import dejavu.intellij.language.psi.DejavuTypes

class DejavuTodoIndexer : LexerBasedTodoIndexer() {
    override fun createLexer(consumer: OccurrenceConsumer): Lexer {
        return object : BaseFilterLexer(DejavuLexer(TemplateConfig()), consumer) {
            override fun advance() {
                if (myDelegate.tokenType == DejavuTypes.COMMENT_CONTENT) {
                    scanWordsInToken(UsageSearchContext.IN_COMMENTS.toInt(), false, false)
                    advanceTodoItemCountsInToken()
                }
                myDelegate.advance()
            }
        }
    }
}
