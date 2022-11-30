package dejavu.intellij.ide.annotator

import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.psi.PsiElement
import com.intellij.psi.util.PsiTreeUtil
import dejavu.intellij.language.DokiLanguage
import dejavu.intellij.language.psi.DejavuTypes
import dejavu.intellij.language.psi.expressions.DotCallNode
import dejavu.intellij.language.psi.expressions.PrimaryExpressionNode
import dejavu.intellij.language.workspace.DokiContextManager

/**
 * Doki框架注解器
 * 负责Doki框架上下文关键词的类型检查
 */
class DokiAnnotator : Annotator {

    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        // 只处理Doki语言的元素
        if (element.language != DokiLanguage) {
            return
        }

        // 检查成员访问表达式（如 post.title）
        val dotCall = PsiTreeUtil.getParentOfType(element, DotCallNode::class.java)
        if (dotCall != null && element.node.elementType == DejavuTypes.IDENTIFIER) {
            // 检查属性是否存在
            checkContextKeywordProperty(dotCall, element, holder)
        }
    }

    /**
     * 检查上下文关键词的属性是否存在
     */
    private fun checkContextKeywordProperty(
        dotCall: DotCallNode,
        propertyElement: PsiElement,
        holder: AnnotationHolder
    ) {
        val project = propertyElement.project
        val contextManager = DokiContextManager.getInstance(project)

        // 找到点号前的表达式，获取上下文关键词名称
        val primaryExpr = PsiTreeUtil.getChildOfType(dotCall, PrimaryExpressionNode::class.java)
        if (primaryExpr != null) {
            val identifier = primaryExpr.firstChild
            if (identifier != null && identifier.node.elementType == DejavuTypes.IDENTIFIER) {
                val keyword = identifier.text
                if (contextManager.isContextKeyword(keyword)) {
                    val propertyName = propertyElement.text
                    if (!contextManager.hasContextKeywordProperty(keyword, propertyName)) {
                        // 标记不存在的属性
                        holder.newAnnotation(
                            HighlightSeverity.ERROR,
                            "Property '$propertyName' does not exist for context keyword '$keyword'"
                        ).range(propertyElement).create()
                    }
                }
            }
        }
    }
}
