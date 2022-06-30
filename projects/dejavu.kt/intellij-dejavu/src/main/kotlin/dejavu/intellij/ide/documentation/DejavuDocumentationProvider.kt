package dejavu.intellij.ide.documentation

import com.intellij.lang.documentation.DocumentationProvider
import com.intellij.psi.PsiElement
import dejavu.intellij.language.psi.DejavuTypes

/**
 * Dejavu 文档提供者
 */
class DejavuDocumentationProvider : DocumentationProvider {
    override fun generateDoc(element: PsiElement, originalElement: PsiElement?): String? {
        return null
    }
}
