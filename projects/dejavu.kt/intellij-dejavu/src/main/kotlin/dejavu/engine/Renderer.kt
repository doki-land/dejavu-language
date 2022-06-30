package dejavu.engine

import com.intellij.openapi.project.Project
import com.intellij.psi.PsiFileFactory
import dejavu.intellij.language.DejavuLanguage
import dejavu.intellij.language.file.DejavuFileNode

class DejavuEngine(private val project: Project) {
    fun render(template: String, data: Map<String, *>): String {
        // 创建临时 PSI 文件
        val psiFile = PsiFileFactory.getInstance(project)
            .createFileFromText("temp.dejavu", DejavuLanguage, template) as DejavuFileNode

        // 使用 PsiRenderer 渲染
        val renderer = PsiRenderer(data)
        return renderer.render(psiFile)
    }
}
