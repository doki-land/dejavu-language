package dejavu.intellij.language.psi

import com.intellij.psi.PsiElementVisitor
import dejavu.intellij.language.psi.expressions.*
import dejavu.intellij.language.psi.fragments.*
import dejavu.intellij.language.psi.templates.*

/**
 * Dejavu 语言 PSI 访问器
 * 遵循 IntelliJ PSI Visitor 设计模式
 */
open class DejavuVisitor : PsiElementVisitor() {

    /**
     * 访问 Fragment 节点
     */
    open fun visitFragmentNode(node: DejavuFragmentNode) {
        visitElement(node)
    }

    /**
     * 访问 BlockFragmentNode
     */
    open fun visitBlockFragmentNode(node: BlockFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 ExtendsFragmentNode
     */
    open fun visitExtendsFragmentNode(node: ExtendsFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 IfFragmentNode
     */
    open fun visitIfFragmentNode(node: IfFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 ElseIfFragmentNode
     */
    open fun visitElseIfFragmentNode(node: ElseIfFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 ElseFragmentNode
     */
    open fun visitElseFragmentNode(node: ElseFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 EndFragmentNode
     */
    open fun visitEndFragmentNode(node: EndFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 LoopFragmentNode
     */
    open fun visitLoopFragmentNode(node: LoopFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 WhileFragmentNode
     */
    open fun visitWhileFragmentNode(node: WhileFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 UntilFragmentNode
     */
    open fun visitUntilFragmentNode(node: UntilFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 MatchFragmentNode
     */
    open fun visitMatchFragmentNode(node: MatchFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 CaseFragmentNode
     */
    open fun visitCaseFragmentNode(node: CaseFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 IncludeFragmentNode
     */
    open fun visitIncludeFragmentNode(node: IncludeFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 RawFragmentNode
     */
    open fun visitRawFragmentNode(node: RawFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 MacroFragmentNode
     */
    open fun visitMacroFragmentNode(node: MacroFragmentNode) {
        visitFragmentNode(node)
    }

    /**
     * 访问 Template 节点
     */
    open fun visitTemplateNode(node: DejavuTemplateNode) {
        visitElement(node)
    }

    /**
     * 访问 BlockTemplateNode
     */
    open fun visitBlockTemplateNode(node: BlockTemplateNode) {
        visitTemplateNode(node)
    }

    /**
     * 访问 IfTemplateNode
     */
    open fun visitIfTemplateNode(node: IfTemplateNode) {
        visitTemplateNode(node)
    }

    /**
     * 访问 LoopTemplateNode
     */
    open fun visitLoopTemplateNode(node: LoopTemplateNode) {
        visitTemplateNode(node)
    }

    /**
     * 访问 WhileTemplateNode
     */
    open fun visitWhileTemplateNode(node: WhileTemplateNode) {
        visitTemplateNode(node)
    }

    /**
     * 访问 MatchTemplateNode
     */
    open fun visitMatchTemplateNode(node: MatchTemplateNode) {
        visitTemplateNode(node)
    }

    /**
     * 访问 RawTemplateNode
     */
    open fun visitRawTemplateNode(node: RawTemplateNode) {
        visitTemplateNode(node)
    }

    /**
     * 访问 ExpressionTemplateNode
     */
    open fun visitExpressionTemplateNode(node: ExpressionTemplateNode) {
        visitTemplateNode(node)
    }

    /**
     * 访问 MacroTemplateNode
     */
    open fun visitMacroTemplateNode(node: MacroTemplateNode) {
        visitTemplateNode(node)
    }

    /**
     * 访问表达式节点
     */
    open fun visitExpressionNode(node: DejavuExpressionNode) {
        visitElement(node)
    }

    /**
     * 访问函数调用节点
     */
    open fun visitFunctionCallNode(node: FunctionCallNode) {
        visitExpressionNode(node)
    }

    /**
     * 访问数组访问节点
     */
    open fun visitArrayAccessNode(node: ArrayCallNode) {
        visitExpressionNode(node)
    }

    /**
     * 访问成员访问节点
     */
    open fun visitMemberAccessNode(node: DotCallNode) {
        visitExpressionNode(node)
    }

    /**
     * 访问二元表达式节点
     */
    open fun visitBinaryExpressionNode(node: BinaryExpressionNode) {
        visitExpressionNode(node)
    }

    /**
     * 访问一元表达式节点
     */
    open fun visitUnaryExpressionNode(node: UnaryExpressionNode) {
        visitExpressionNode(node)
    }

    /**
     * 访问基本表达式节点
     */
    open fun visitPrimaryExpressionNode(node: PrimaryExpressionNode) {
        visitExpressionNode(node)
    }
}
