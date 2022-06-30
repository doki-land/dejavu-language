package dejavu.intellij.language.psi

import com.intellij.lang.ASTNode
import com.intellij.psi.PsiElement
import dejavu.intellij.language.psi.fragments.*
import dejavu.intellij.language.psi.expressions.*
import dejavu.intellij.language.psi.templates.*

object DejavuFactory {
    fun createElement(node: ASTNode): PsiElement {
        return when (node.elementType) {
            // Templates
            DejavuTypes.IF_TEMPLATE -> IfTemplateNode(node)
            DejavuTypes.LOOP_TEMPLATE -> LoopTemplateNode(node)
            DejavuTypes.WHILE_TEMPLATE -> WhileTemplateNode(node)
            DejavuTypes.MATCH_TEMPLATE -> MatchTemplateNode(node)
            DejavuTypes.BLOCK_TEMPLATE -> BlockTemplateNode(node)
            DejavuTypes.RAW_TEMPLATE -> RawTemplateNode(node)
            DejavuTypes.EXPRESSION_TEMPLATE -> ExpressionTemplateNode(node)
            DejavuTypes.MACRO_TEMPLATE -> MacroTemplateNode(node)

            // Fragments
            DejavuTypes.IF_FRAGMENT -> IfFragmentNode(node)
            DejavuTypes.ELSE_IF_FRAGMENT -> ElseIfFragmentNode(node)
            DejavuTypes.ELSE_FRAGMENT -> ElseFragmentNode(node)
            DejavuTypes.END_FRAGMENT -> EndFragmentNode(node)
            DejavuTypes.LOOP_FRAGMENT -> LoopFragmentNode(node)
            DejavuTypes.WHILE_FRAGMENT -> WhileFragmentNode(node)
            DejavuTypes.UNTIL_FRAGMENT -> UntilFragmentNode(node)
            DejavuTypes.MATCH_FRAGMENT -> MatchFragmentNode(node)
            DejavuTypes.CASE_FRAGMENT -> CaseFragmentNode(node)
            DejavuTypes.BLOCK_FRAGMENT -> BlockFragmentNode(node)
            DejavuTypes.EXTENDS_FRAGMENT -> ExtendsFragmentNode(node)
            DejavuTypes.INCLUDE_FRAGMENT -> IncludeFragmentNode(node)
            DejavuTypes.RAW_FRAGMENT -> RawFragmentNode(node)
            DejavuTypes.MACRO_FRAGMENT -> MacroFragmentNode(node)

            // Other elements
            DejavuTypes.COMMENT -> DejavuCommentElement(node)
            DejavuTypes.TEXT -> DejavuTextLiteralElement(node)

            // Expressions
            DejavuTypes.FUNCTION_CALL -> FunctionCallNode(node)
            DejavuTypes.ARRAY_ACCESS -> ArrayCallNode(node)
            DejavuTypes.MEMBER_ACCESS -> DotCallNode(node)
            DejavuTypes.BINARY_EXPRESSION -> BinaryExpressionNode(node)
            DejavuTypes.UNARY_EXPRESSION -> UnaryExpressionNode(node)
            DejavuTypes.PRIMARY_EXPRESSION -> PrimaryExpressionNode(node)

            else -> DejavuElement(node)
        }
    }
}
