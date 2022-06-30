package dejavu.intellij.ide.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.PlatformDataKeys
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.Messages
import com.intellij.openapi.vfs.VirtualFile
import dejavu.engine.DejavuEngine
import dejavu.intellij.language.file.DejavuBundle
import com.google.gson.Gson
import com.google.gson.reflect.TypeToken
import java.io.File
import java.io.IOException

class PreviewTemplateAction :
    AnAction(DejavuBundle.message("action.preview.title"), DejavuBundle.message("action.preview.description"), null) {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val virtualFile = e.getData(PlatformDataKeys.VIRTUAL_FILE) ?: return

        if (virtualFile.extension != "dejavu") {
            Messages.showErrorDialog(
                project,
                DejavuBundle.message("error.not.dejavu.file"),
                DejavuBundle.message("error.title")
            )
            return
        }

        try {
            // 读取模板文件
            val templateContent = virtualFile.inputStream.bufferedReader().use { it.readText() }

            // 读取数据文件（同名的 .data.json）
            val dataFilePath = virtualFile.path.replace(Regex("\\.dejavu$"), ".data.json")
            val dataFile = File(dataFilePath)
            val data = if (dataFile.exists()) {
                try {
                    val jsonContent = dataFile.readText()
                    val gson = Gson()
                    val type = object : TypeToken<Map<String, Any>>() {}.type
                    gson.fromJson(jsonContent, type)
                } catch (e: Exception) {
                    Messages.showErrorDialog(
                        project,
                        DejavuBundle.message("error.invalid.data.file"),
                        DejavuBundle.message("error.title")
                    )
                    return
                }
            } else {
                emptyMap<String, Any>()
            }

            // 渲染模板
            val engine = DejavuEngine(project)
            val renderedContent = engine.render(templateContent, data as Map<String, *>)

            // 显示预览结果
            Messages.showInfoMessage(project, renderedContent, DejavuBundle.message("preview.title"))
        } catch (e: IOException) {
            Messages.showErrorDialog(
                project,
                DejavuBundle.message("error.reading.file", e.message ?: "Unknown error"),
                DejavuBundle.message("error.title")
            )
        }
    }

    override fun update(e: AnActionEvent) {
        val project = e.project
        val virtualFile = e.getData(PlatformDataKeys.VIRTUAL_FILE)

        e.presentation.isEnabledAndVisible = project != null && virtualFile != null &&
                virtualFile.extension == "dejavu"
    }
}
