package dejavu.intellij.ide

import com.intellij.codeInsight.editorActions.QuoteHandler
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.editor.highlighter.HighlighterIterator

class DejavuQuoteHandler : QuoteHandler {
    override fun isClosingQuote(iterator: HighlighterIterator?, offset: Int): Boolean {
        TODO("Not yet implemented")
    }

    override fun isOpeningQuote(iterator: HighlighterIterator?, offset: Int): Boolean {
        TODO("Not yet implemented")
    }

    override fun hasNonClosedLiteral(editor: Editor?, iterator: HighlighterIterator?, offset: Int): Boolean {
        TODO("Not yet implemented")
    }

    override fun isInsideLiteral(iterator: HighlighterIterator?): Boolean {
        TODO("Not yet implemented")
    }
}
