package dejavu.intellij.language.psi.templates

import com.intellij.lang.ASTNode
import com.intellij.psi.PsiElementVisitor
import dejavu.intellij.language.psi.DejavuElement
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * Template 节点的基类
 */
abstract class DejavuTemplateNode(node: ASTNode) : DejavuElement(node) {

    override fun accept(visitor: PsiElementVisitor) {
        if (visitor is DejavuVisitor) {
            accept(visitor)
        } else {
            super.accept(visitor)
        }
    }

    /**
     * 接受 DejavuVisitor 访问
     */
    abstract fun accept(visitor: DejavuVisitor)
}
