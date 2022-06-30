package dejavu.engine

import com.intellij.psi.PsiElement
import dejavu.intellij.language.psi.*
import dejavu.intellij.language.psi.expressions.*
import dejavu.intellij.language.psi.fragments.*
import dejavu.intellij.language.psi.templates.*

class PsiRenderer(private val data: Map<String, *>) {
    fun render(element: PsiElement): String {
        val visitor = object : DejavuRecursiveVisitor() {
            val result = StringBuilder()

            override fun visitElement(element: PsiElement) {
                when (element) {
                    is DejavuTextLiteralElement -> result.append(element.text)
                    is IfTemplateNode -> visitIfTemplate(element)
                    is LoopTemplateNode -> visitLoopTemplate(element)
                    is BlockFragmentNode -> visitBlockFragment(element)
                    is DejavuExpressionNode -> result.append(evaluateExpression(element))
                    else -> super.visitElement(element)
                }
            }

            private fun visitIfTemplate(element: IfTemplateNode) {
                val condition = findExpression(element)
                if (condition != null && evaluateExpression(condition) as? Boolean == true) {
                    element.children.forEach {
                        if (it !is DejavuExpressionNode) {
                            it.accept(this)
                        }
                    }
                }
            }

            private fun visitLoopTemplate(element: LoopTemplateNode) {
                val arrayExpr = findExpression(element)
                val array = evaluateExpression(arrayExpr) as? List<*>
                if (array != null) {
                    val variableName = findIdentifier(element)?.text ?: error("Loop template must have a variable name")
                    array.forEach { item ->
                        val itemData = if (item is Map<*, *>) {
                            item.entries.associate { (k, v) ->
                                (k as? String ?: k.toString()) to v
                            }
                        } else {
                            mapOf(variableName to item)
                        }
                        val nestedRenderer = PsiRenderer(itemData)
                        element.children.forEach {
                            if (it !is DejavuExpressionNode && !isIdentifier(it)) {
                                result.append(nestedRenderer.render(it))
                            }
                        }
                    }
                }
            }

            private fun visitBlockFragment(element: BlockFragmentNode) {
                element.children.forEach {
                    it.accept(this)
                }
            }

            private fun findExpression(element: PsiElement): PsiElement? {
                return element.children.find { it is DejavuExpressionNode }
            }

            private fun findIdentifier(element: PsiElement): PsiElement? {
                return element.children.find { it.node.elementType == DejavuTypes.IDENTIFIER }
            }

            private fun isIdentifier(element: PsiElement): Boolean {
                return element.node.elementType == DejavuTypes.IDENTIFIER
            }

            private fun evaluateExpression(expression: PsiElement?): Any {
                if (expression == null) return false
                return when {
                    isIdentifier(expression) -> data[expression.text] ?: false
                    expression is DotCallNode -> evaluateMemberAccess(expression)
                    expression is FunctionCallNode -> evaluateFunctionCall(expression)
                    expression is ArrayCallNode -> evaluateArrayAccess(expression)
                    isStringLiteral(expression) -> expression.text.substring(1, expression.text.length - 1)
                    isNumberLiteral(expression) -> expression.text.toDoubleOrNull() ?: 0.0
                    isBooleanLiteral(expression) -> expression.text.toBoolean()
                    else -> false
                }
            }

            private fun evaluateMemberAccess(expression: DotCallNode): Any {
                val objectExpr = expression.firstChild ?: error("Dot call must have an object expression")
                val objectValue = evaluateExpression(objectExpr)
                if (objectValue is Map<*, *>) {
                    val propertyName = expression.lastChild?.text ?: error("Dot call must have a property name")
                    return objectValue[propertyName] ?: false
                }
                return false
            }

            private fun evaluateFunctionCall(expression: FunctionCallNode): Any {
                // 简单实现，实际需要解析函数调用表达式
                return false
            }

            private fun evaluateArrayAccess(expression: ArrayCallNode): Any {
                // 简单实现，实际需要解析数组访问表达式
                return false
            }

            private fun isStringLiteral(element: PsiElement): Boolean {
                return element.node.elementType == DejavuTypes.STRING
            }

            private fun isNumberLiteral(element: PsiElement): Boolean {
                return element.node.elementType == DejavuTypes.NUMBER
            }

            private fun isBooleanLiteral(element: PsiElement): Boolean {
                val text = element.text
                return text == "true" || text == "false"
            }
        }

        element.accept(visitor)
        return visitor.result.toString()
    }
}
