package dejavu.intellij.ide.annotator

import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.psi.PsiElement
import dejavu.intellij.ide.highlight.HighlightColor

/**
 * Dejavu 上下文相关高亮处理器
 */
class DejavuContextualHighlightingAnnotator : Annotator {
    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        // 暂时不处理任何上下文高亮
    }
}
