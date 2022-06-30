package dejavu.intellij.language.psi.templates

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * Loop template 节点
 */
class LoopTemplateNode(node: ASTNode) : DejavuTemplateNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitLoopTemplateNode(this)
    }
}
