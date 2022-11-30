package dejavu.intellij.language

/**
 * 模板配置类，支持多种定界符和空白控制符
 */
data class TemplateConfig(
    // 基本定界符
    val slotStart: String = "<%",
    val slotEnd: String = "%>",
    val commentStart: String = "<#",
    val commentEnd: String = "#>",
    val allowPipeOperator: Boolean = false,
    val allowLegacyFor: Boolean = false,
)