package dejavu.intellij.language.psi.templates

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * While template 节点
 */
class WhileTemplateNode(node: ASTNode) : DejavuTemplateNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitWhileTemplateNode(this)
    }
}
