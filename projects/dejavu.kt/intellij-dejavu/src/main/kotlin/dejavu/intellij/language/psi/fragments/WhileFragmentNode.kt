package dejavu.intellij.language.psi.fragments

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * While fragment 节点
 */
class WhileFragmentNode(node: ASTNode) : DejavuFragmentNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitWhileFragmentNode(this)
    }
}
