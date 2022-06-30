package dejavu.intellij.language.psi.fragments

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * Raw fragment 节点
 */
class RawFragmentNode(node: ASTNode) : DejavuFragmentNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitRawFragmentNode(this)
    }
}
