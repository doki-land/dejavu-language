package dejavu.intellij.ide.completion

import com.intellij.codeInsight.completion.CompletionParameters
import com.intellij.codeInsight.completion.CompletionProvider
import com.intellij.codeInsight.completion.CompletionResultSet
import com.intellij.codeInsight.lookup.LookupElementBuilder
import com.intellij.openapi.editor.EditorModificationUtil
import com.intellij.util.ProcessingContext
import dejavu.intellij.language.file.DejavuIconProvider
import javax.swing.Icon

class TextCompletionProvider : CompletionProvider<CompletionParameters>() {
    override fun addCompletions(
        parameters: CompletionParameters, context: ProcessingContext, resultSet: CompletionResultSet
    ) {
        resultSet.addSlot(
            "loop-i-slot", "loop i in items", """
            <% loop i in body %>
                
            <% end loop %>
            """, -10
        )
        resultSet.addSlot(
            "loop-kv-slot", "loop k, v in items", """
            <% loop k, v in body %>
                
            <% end loop %>
            """, -10
        )
        resultSet.addSlot(
            "loop-else-slot", "loop .. else", """
            <% loop i in body %>
                
            <% else %>
                
            <% end loop %>
            """, -10
        )
        resultSet.addInline(
            "loop-i-inline", "loop i in items", """
            <% loop i in body %>
                
            <% end loop %>
            """
        )
        resultSet.addInline(
            "loop-kv-inline", "loop k, v in items", """
            <% loop k, v in body %>
                
            <% end loop %>
            """
        )
        resultSet.addSlot(
            "if-slot", "if", """
            <% if condition %>
                
            <% end %>
            """, -10
        )
        resultSet.addInline(
            "if-inline", "if", """
            <% if condition {} %>
            """
        )
        resultSet.addSlot(
            "else", "else", """
            <% else %>
            """
        )
        resultSet.addElement(
            "variable", "variable", """
            <%  %>
            """
        )

        resultSet.addElement(
            "super", "super", """
            <% super %>
            """
        )

        resultSet.addElement(
            "looper", "looper", """
            <% looper %>
            """
        )
    }

    private fun CompletionResultSet.addSlot(
        keys: String, showText: String, insertText: String, shift: Int = 0
    ) {
        this.addElement(keys, "<% $showText %>", insertText, shift = shift)
    }

    private fun CompletionResultSet.addInline(
        keys: String, showText: String, insertText: String, shift: Int = 0
    ) {
        this.addElement(keys, showText, insertText, shift = shift)
    }

    private fun CompletionResultSet.addElement(
        keys: String,
        showText: String,
        insertText: String,
        typeText: String? = null,
        tailText: String? = null,
        icon: Icon? = null,
        shift: Int = 0
    ) {
        val e = LookupElementBuilder.create(keys).withCaseSensitivity(false).withPresentableText(showText)
            .withTailText(tailText, true).withTypeText(typeText).withIcon(icon).withInsertHandler { context, _ ->
                val document = context.document
                document.replaceString(context.startOffset, context.tailOffset, insertText.trimIndent())
                EditorModificationUtil.moveCaretRelatively(context.editor, shift)
            };
        this.addElement(e)
    }
}
