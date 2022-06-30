package dejavu.intellij.language.psi

import com.intellij.extapi.psi.ASTWrapperPsiElement
import com.intellij.lang.ASTNode

/**
 * Dejavu 语言 PSI 元素的基类
 */
open class DejavuElement(node: ASTNode) : ASTWrapperPsiElement(node) {
    override fun toString(): String {
        return "${this::class.simpleName}(${node.elementType})"
    }
}
