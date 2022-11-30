package dejavu.intellij.ide.annotator

import com.intellij.lang.annotation.AnnotationHolder
import com.intellij.lang.annotation.Annotator
import com.intellij.psi.PsiElement
import com.intellij.psi.util.PsiTreeUtil
import dejavu.intellij.language.psi.DejavuTypes

/**
 * Dejavu 错误检查器
 * 为后续完整实现预留接口
 */
class DejavuAnnotator : Annotator {
    override fun annotate(element: PsiElement, holder: AnnotationHolder) {
        // 桩实现：目前不做任何检查
        // 未来实现：
        // 1. 检查未闭合的模板标签
        // 2. 检查语法错误
        // 3. 检查变量未定义
        // 4. 检查类型不匹配
    }
}
