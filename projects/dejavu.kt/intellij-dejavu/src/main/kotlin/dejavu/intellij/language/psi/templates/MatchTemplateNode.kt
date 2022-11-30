package dejavu.intellij.language.psi.templates

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * Match template 节点
 */
class MatchTemplateNode(node: ASTNode) : DejavuTemplateNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitMatchTemplateNode(this)
    }
}
