package dejavu.intellij.ide.refactoring

import com.intellij.psi.PsiElement
import com.intellij.refactoring.rename.RenamePsiElementProcessor
import com.intellij.usageView.UsageInfo
import com.intellij.refactoring.listeners.RefactoringElementListener

/**
 * Dejavu 重命名处理器
 * 为后续完整实现预留接口
 */
class DejavuRenameProcessor : RenamePsiElementProcessor() {
    override fun canProcessElement(element: PsiElement): Boolean {
        // 桩实现：返回 false 表示不处理任何元素
        // 未来实现：返回 true 表示可以处理 Dejavu 特定元素（如变量、函数等）
        return false
    }

    override fun renameElement(
        element: PsiElement,
        newName: String,
        usages: Array<UsageInfo>,
        listener: RefactoringElementListener?
    ) {
        // 桩实现：不做任何处理
        // 未来实现：
        // 1. 分析要重命名的元素类型
        // 2. 在所有引用处更新名称
        // 3. 处理注释和字符串中的引用
    }
}
