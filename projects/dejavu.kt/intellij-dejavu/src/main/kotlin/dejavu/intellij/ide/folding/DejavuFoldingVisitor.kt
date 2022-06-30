package dejavu.intellij.ide.folding

import com.intellij.lang.folding.FoldingDescriptor
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiComment
import com.intellij.psi.PsiElement
import dejavu.intellij.language.psi.DejavuRecursiveVisitor
import dejavu.intellij.language.psi.DejavuTypes

class DejavuFoldingVisitor(private val descriptors: MutableList<FoldingDescriptor>) : DejavuRecursiveVisitor() {

    override fun visitComment(comment: PsiComment) {
        if (comment.tokenType == DejavuTypes.COMMENT_CONTENT) {
            fold(comment)
            super.visitComment(comment)
        }
    }

    private fun fold(element: PsiElement) {
        descriptors += FoldingDescriptor(element.node, element.textRange)
    }

    private fun fold(element: PsiElement, placeholder: String = "...", collapse: Boolean = false) {
        descriptors += FoldingDescriptor(element.node, element.textRange, null, setOf(), false, placeholder, collapse)
    }

    private fun fold(
        element: PsiElement,
        start: Int?,
        end: Int?,
        placeholder: String = "...",
        collapse: Boolean = false
    ) {
        if (start != null && end != null && start < end) {
            val range = TextRange(start, end)
            descriptors += FoldingDescriptor(
                element.node, range, null, setOf(), false, placeholder, collapse
            )
        }
    }
}
