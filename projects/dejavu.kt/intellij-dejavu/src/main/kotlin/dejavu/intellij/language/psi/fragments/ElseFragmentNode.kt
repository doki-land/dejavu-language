package dejavu.intellij.language.psi.fragments

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * Else fragment 节点
 */
class ElseFragmentNode(node: ASTNode) : DejavuFragmentNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitElseFragmentNode(this)
    }
}
