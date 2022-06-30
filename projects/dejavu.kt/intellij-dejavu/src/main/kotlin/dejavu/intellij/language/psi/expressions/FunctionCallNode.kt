package dejavu.intellij.language.psi.expressions

import com.intellij.lang.ASTNode
import dejavu.intellij.language.psi.DejavuVisitor

/**
 * 函数调用表达式节点 `f(args)`
 */
class FunctionCallNode(node: ASTNode) : DejavuExpressionNode(node) {

    override fun accept(visitor: DejavuVisitor) {
        visitor.visitFunctionCallNode(this)
    }
}
