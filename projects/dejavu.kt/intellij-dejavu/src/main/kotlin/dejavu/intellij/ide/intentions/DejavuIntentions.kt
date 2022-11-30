package dejavu.intellij.ide.intentions

import com.intellij.codeInsight.intention.IntentionAction
import com.intellij.codeInsight.intention.PsiElementBaseIntentionAction
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.project.Project
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.util.IncorrectOperationException

/**
 * Dejavu 意图动作
 * 为后续完整实现预留接口
 */
class DejavuCreateVariableIntention : PsiElementBaseIntentionAction(), IntentionAction {
    override fun getText(): String {
        return "Create Dejavu variable"
    }

    override fun getFamilyName(): String {
        return "Dejavu"
    }

    override fun isAvailable(project: Project, editor: Editor, element: PsiElement): Boolean {
        // 桩实现：返回 false 表示在任何位置都不可用
        // 未来实现：返回 true 表示在适当位置可用（如表达式中）
        return false
    }

    @Throws(IncorrectOperationException::class)
    override fun invoke(project: Project, editor: Editor, element: PsiElement) {
        // 桩实现：不做任何处理
        // 未来实现：创建变量声明
    }
}

class DejavuAddMissingEndIntention : PsiElementBaseIntentionAction(), IntentionAction {
    override fun getText(): String {
        return "Add missing end segment"
    }

    override fun getFamilyName(): String {
        return "Dejavu"
    }

    override fun isAvailable(project: Project, editor: Editor, element: PsiElement): Boolean {
        // 桩实现：返回 false 表示在任何位置都不可用
        // 未来实现：返回 true 表示在缺少 end 段的位置可用
        return false
    }

    @Throws(IncorrectOperationException::class)
    override fun invoke(project: Project, editor: Editor, element: PsiElement) {
        // 桩实现：不做任何处理
        // 未来实现：添加缺失的 end 段
    }
}
