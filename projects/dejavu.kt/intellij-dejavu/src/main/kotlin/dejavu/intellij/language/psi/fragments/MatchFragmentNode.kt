package dejavu.intellij.language.psi.fragments

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * Match fragment 节点
 */
class MatchFragmentNode(node: ASTNode) : DejavuFragmentNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitMatchFragmentNode(this)
    }
}
