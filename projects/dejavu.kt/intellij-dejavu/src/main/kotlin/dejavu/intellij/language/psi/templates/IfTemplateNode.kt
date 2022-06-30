package dejavu.intellij.language.psi.templates

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * If template 节点
 */
class IfTemplateNode(node: ASTNode) : DejavuTemplateNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitIfTemplateNode(this)
    }
}
