package dejavu.intellij.ide.actions

import com.intellij.ide.actions.CreateFileFromTemplateAction
import com.intellij.ide.actions.CreateFileFromTemplateDialog
import com.intellij.openapi.project.Project
import com.intellij.psi.PsiDirectory
import dejavu.intellij.language.DejavuLanguage
import dejavu.intellij.language.file.DejavuIconProvider
import dejavu.intellij.language.file.DejavuBundle

class CreateDejavuFile : CreateFileFromTemplateAction(Name, Description, DejavuIconProvider.DejavuIcon) {
    companion object {
        val Name = DejavuLanguage.id
        const val TemplateName = "Dejavu File";
        val Description = DejavuBundle.message("action.create.description")
    }

    override fun buildDialog(project: Project, directory: PsiDirectory, builder: CreateFileFromTemplateDialog.Builder) {
        builder.setTitle(Name).addKind("Empty file", DejavuIconProvider.DejavuIcon, TemplateName)
    }

    override fun getActionName(directory: PsiDirectory, newName: String, templateName: String): String = Name
}
