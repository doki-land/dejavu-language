package dejavu.intellij.language.file

import com.intellij.openapi.fileTypes.LanguageFileType
import dejavu.intellij.language.DejavuLanguage
import javax.swing.Icon

object DejavuFileType : LanguageFileType(DejavuLanguage) {
    override fun getName(): String = "Dejavu"

    override fun getDescription(): String = "Dejavu Template File"

    override fun getDefaultExtension(): String = "dejavu"

    override fun getIcon(): Icon? = DejavuIconProvider.DejavuIcon
}