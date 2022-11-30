package dejavu.intellij.language.psi.expressions

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * 基本表达式节点
 */
class PrimaryExpressionNode(node: ASTNode) : DejavuExpressionNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitPrimaryExpressionNode(this)
    }
}
