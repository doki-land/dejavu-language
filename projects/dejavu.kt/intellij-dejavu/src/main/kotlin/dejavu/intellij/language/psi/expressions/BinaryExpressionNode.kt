package dejavu.intellij.language.psi.expressions

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * 二元表达式节点
 */
class BinaryExpressionNode(node: ASTNode) : DejavuExpressionNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitBinaryExpressionNode(this)
    }
}
