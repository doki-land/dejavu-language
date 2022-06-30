package dejavu.intellij.language.file

import com.intellij.openapi.fileTypes.LanguageFileType
import dejavu.intellij.language.DokiLanguage
import javax.swing.Icon

object DokiFileType : LanguageFileType(DokiLanguage) {
    override fun getName(): String = "Doki"

    override fun getDescription(): String = "Doki Template File"

    override fun getDefaultExtension(): String = "doki"

    override fun getIcon(): Icon? = DejavuIconProvider.DejavuIcon
}