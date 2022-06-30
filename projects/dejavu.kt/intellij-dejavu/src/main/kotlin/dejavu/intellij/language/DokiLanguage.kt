package dejavu.intellij.language

import com.intellij.lang.Language

object DokiLanguage : Language("Doki") {
    const val FileExtension = "doki"
    val LanguageConfig = TemplateConfig("{%", "%}", "{#", "#}", allowPipeOperator = true)
}