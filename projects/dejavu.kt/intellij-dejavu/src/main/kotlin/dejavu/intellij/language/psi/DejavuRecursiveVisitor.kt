package dejavu.intellij.language.psi

import com.intellij.psi.PsiElement
import dejavu.intellij.language.psi.expressions.*
import dejavu.intellij.language.psi.fragments.*
import dejavu.intellij.language.psi.templates.*

/**
 * Dejavu 递归访问器
 * 继承 DejavuVisitor，在访问每个节点后递归访问子元素
 */
open class DejavuRecursiveVisitor : DejavuVisitor() {

    override fun visitElement(element: PsiElement) {
        super.visitElement(element)
        // 递归访问子元素
        var child = element.firstChild
        while (child != null) {
            child.accept(this)
            child = child.nextSibling
        }
    }

    override fun visitFragmentNode(node: DejavuFragmentNode) {
        super.visitFragmentNode(node)
        visitElement(node)
    }

    override fun visitBlockFragmentNode(node: BlockFragmentNode) {
        super.visitBlockFragmentNode(node)
        visitElement(node)
    }

    override fun visitExtendsFragmentNode(node: ExtendsFragmentNode) {
        super.visitExtendsFragmentNode(node)
        visitElement(node)
    }

    override fun visitIfFragmentNode(node: IfFragmentNode) {
        super.visitIfFragmentNode(node)
        visitElement(node)
    }

    override fun visitElseIfFragmentNode(node: ElseIfFragmentNode) {
        super.visitElseIfFragmentNode(node)
        visitElement(node)
    }

    override fun visitElseFragmentNode(node: ElseFragmentNode) {
        super.visitElseFragmentNode(node)
        visitElement(node)
    }

    override fun visitEndFragmentNode(node: EndFragmentNode) {
        super.visitEndFragmentNode(node)
        visitElement(node)
    }

    override fun visitLoopFragmentNode(node: LoopFragmentNode) {
        super.visitLoopFragmentNode(node)
        visitElement(node)
    }

    override fun visitWhileFragmentNode(node: WhileFragmentNode) {
        super.visitWhileFragmentNode(node)
        visitElement(node)
    }

    override fun visitUntilFragmentNode(node: UntilFragmentNode) {
        super.visitUntilFragmentNode(node)
        visitElement(node)
    }

    override fun visitMatchFragmentNode(node: MatchFragmentNode) {
        super.visitMatchFragmentNode(node)
        visitElement(node)
    }

    override fun visitCaseFragmentNode(node: CaseFragmentNode) {
        super.visitCaseFragmentNode(node)
        visitElement(node)
    }

    override fun visitIncludeFragmentNode(node: IncludeFragmentNode) {
        super.visitIncludeFragmentNode(node)
        visitElement(node)
    }

    override fun visitRawFragmentNode(node: RawFragmentNode) {
        super.visitRawFragmentNode(node)
        visitElement(node)
    }

    override fun visitMacroFragmentNode(node: MacroFragmentNode) {
        super.visitMacroFragmentNode(node)
        visitElement(node)
    }

    override fun visitTemplateNode(node: DejavuTemplateNode) {
        super.visitTemplateNode(node)
        visitElement(node)
    }

    override fun visitBlockTemplateNode(node: BlockTemplateNode) {
        super.visitBlockTemplateNode(node)
        visitElement(node)
    }

    override fun visitIfTemplateNode(node: IfTemplateNode) {
        super.visitIfTemplateNode(node)
        visitElement(node)
    }

    override fun visitLoopTemplateNode(node: LoopTemplateNode) {
        super.visitLoopTemplateNode(node)
        visitElement(node)
    }

    override fun visitWhileTemplateNode(node: WhileTemplateNode) {
        super.visitWhileTemplateNode(node)
        visitElement(node)
    }

    override fun visitMatchTemplateNode(node: MatchTemplateNode) {
        super.visitMatchTemplateNode(node)
        visitElement(node)
    }

    override fun visitRawTemplateNode(node: RawTemplateNode) {
        super.visitRawTemplateNode(node)
        visitElement(node)
    }

    override fun visitExpressionTemplateNode(node: ExpressionTemplateNode) {
        super.visitExpressionTemplateNode(node)
        visitElement(node)
    }

    override fun visitMacroTemplateNode(node: MacroTemplateNode) {
        super.visitMacroTemplateNode(node)
        visitElement(node)
    }

    override fun visitExpressionNode(node: DejavuExpressionNode) {
        super.visitExpressionNode(node)
        visitElement(node)
    }

    override fun visitFunctionCallNode(node: FunctionCallNode) {
        super.visitFunctionCallNode(node)
        visitElement(node)
    }

    override fun visitArrayAccessNode(node: ArrayCallNode) {
        super.visitArrayAccessNode(node)
        visitElement(node)
    }

    override fun visitMemberAccessNode(node: DotCallNode) {
        super.visitMemberAccessNode(node)
        visitElement(node)
    }

    override fun visitBinaryExpressionNode(node: BinaryExpressionNode) {
        super.visitBinaryExpressionNode(node)
        visitElement(node)
    }

    override fun visitUnaryExpressionNode(node: UnaryExpressionNode) {
        super.visitUnaryExpressionNode(node)
        visitElement(node)
    }

    override fun visitPrimaryExpressionNode(node: PrimaryExpressionNode) {
        super.visitPrimaryExpressionNode(node)
        visitElement(node)
    }
}
