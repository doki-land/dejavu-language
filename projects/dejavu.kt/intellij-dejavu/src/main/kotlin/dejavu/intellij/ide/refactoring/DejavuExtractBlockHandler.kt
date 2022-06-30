package dejavu.intellij.ide.refactoring

import com.intellij.openapi.actionSystem.DataContext
import com.intellij.openapi.command.WriteCommandAction
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.editor.ScrollType
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.Messages
import com.intellij.openapi.util.TextRange
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiFile
import com.intellij.psi.util.PsiTreeUtil
import com.intellij.refactoring.RefactoringActionHandler
import com.intellij.refactoring.util.CommonRefactoringUtil
import dejavu.intellij.language.psi.DejavuTypes
import dejavu.intellij.language.file.DejavuFileType

/**
 * Extract Block 重构处理器
 *
 * 将选中的代码块提取为一个新的 block 定义
 */
class DejavuExtractBlockHandler : RefactoringActionHandler {

    override fun invoke(project: Project, editor: Editor, file: PsiFile, dataContext: DataContext) {
        if (file.fileType != DejavuFileType) {
            return
        }

        val selectionModel = editor.selectionModel
        if (!selectionModel.hasSelection()) {
            CommonRefactoringUtil.showErrorHint(
                project,
                editor,
                "Please select the code block to extract",
                "Extract Block",
                null
            )
            return
        }

        val startOffset = selectionModel.selectionStart
        val endOffset = selectionModel.selectionEnd
        val selectedText = selectionModel.selectedText ?: return

        // 获取 block 名称
        val blockName = Messages.showInputDialog(
            project,
            "Enter block name:",
            "Extract Block",
            Messages.getQuestionIcon(),
            "newBlock",
            null
        ) ?: return

        if (blockName.isBlank()) {
            return
        }

        performExtractBlock(project, editor, file, startOffset, endOffset, selectedText, blockName)
    }

    override fun invoke(project: Project, elements: Array<out PsiElement>, dataContext: DataContext) {
        // 不支持从元素直接调用
    }

    private fun performExtractBlock(
        project: Project,
        editor: Editor,
        file: PsiFile,
        startOffset: Int,
        endOffset: Int,
        selectedText: String,
        blockName: String
    ) {
        WriteCommandAction.runWriteCommandAction(project) {
            val document = editor.document

            // 构建替换文本
            val blockCall = "<% block $blockName %>"
            val blockEnd = "<% end %>"

            // 替换选中的文本为 block 调用
            val replacementText = "$blockCall\n$selectedText\n$blockEnd"

            // 执行替换
            document.replaceString(startOffset, endOffset, replacementText)

            // 重新格式化
            com.intellij.psi.codeStyle.CodeStyleManager.getInstance(project)
                .reformatText(file, startOffset, startOffset + replacementText.length)

            // 滚动到修改位置
            editor.scrollingModel.scrollToCaret(ScrollType.MAKE_VISIBLE)
        }
    }
}
