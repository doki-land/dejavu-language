package dejavu.intellij.ide.highlight

import com.intellij.openapi.options.colors.ColorDescriptor
import com.intellij.openapi.options.colors.ColorSettingsPage
import dejavu.intellij.language.file.DejavuBundle
import dejavu.intellij.language.file.DejavuIconProvider

class DejavuHighlightSettingPage : ColorSettingsPage {
    private val attrs = HighlightColor
        .values()
        .map { it.attributesDescriptor }
        .toTypedArray()

    private val annotatorTags = HighlightColor
        .values()
        .associateBy({ it.name }, { it.textAttributesKey })

    override fun getAttributeDescriptors() = attrs

    override fun getColorDescriptors(): Array<ColorDescriptor> = ColorDescriptor.EMPTY_ARRAY

    override fun getDisplayName() = DejavuBundle.message("filetype.name")

    override fun getIcon() = DejavuIconProvider.DejavuIcon

    override fun getHighlighter() = DejavuSyntaxHighlighter()

    override fun getDemoText() = javaClass.getResource("/fileTemplates/demoColor.html")?.readText() ?: ""
    override fun getAdditionalHighlightingTagToDescriptorMap() = annotatorTags
}
