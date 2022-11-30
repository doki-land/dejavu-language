package dejavu.intellij.ide.refactoring

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.project.Project

/**
 * Extract Block 动作
 *
 * 包装 DejavuExtractBlockHandler 的 AnAction
 */
class DejavuExtractBlockAction : AnAction("Extract Block", "Extract selected code into a block", null) {

    private val handler = DejavuExtractBlockHandler()

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val editor = e.getData(CommonDataKeys.EDITOR) ?: return
        val file = e.getData(CommonDataKeys.PSI_FILE) ?: return

        handler.invoke(project, editor, file, e.dataContext)
    }

    override fun update(e: AnActionEvent) {
        val project = e.project
        val editor = e.getData(CommonDataKeys.EDITOR)
        val file = e.getData(CommonDataKeys.PSI_FILE)

        // 只在 Dejavu 文件中启用
        e.presentation.isEnabledAndVisible = project != null
                && editor != null
                && file != null
                && file.fileType.name == "Dejavu"
    }
}
