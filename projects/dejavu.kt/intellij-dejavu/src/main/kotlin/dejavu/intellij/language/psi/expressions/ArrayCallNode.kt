package dejavu.intellij.language.psi.expressions

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * 数组访问表达式节点 `f[index]`
 */
class ArrayCallNode(node: ASTNode) : DejavuExpressionNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitArrayAccessNode(this)
    }
}
