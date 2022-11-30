package dejavu.intellij.language.psi.expressions

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * 成员访问表达式节点 `f.x`
 */
class DotCallNode(node: ASTNode) : DejavuExpressionNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitMemberAccessNode(this)
    }
}
