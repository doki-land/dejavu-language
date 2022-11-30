package dejavu.intellij.language

import com.intellij.lang.Language

object DejavuLanguage : Language("Dejavu") {
    const val Bundle = "messages.DejavuBundle"
    const val FileExtension = "dj;dejavu;"
    val LanguageConfig = TemplateConfig("<%", "%>", "<#", "#>")
}

