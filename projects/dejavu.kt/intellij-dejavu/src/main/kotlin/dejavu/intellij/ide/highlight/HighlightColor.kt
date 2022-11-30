package dejavu.intellij.ide.highlight

import com.intellij.lang.annotation.HighlightSeverity
import com.intellij.openapi.editor.HighlighterColors
import com.intellij.openapi.editor.colors.TextAttributesKey
import com.intellij.openapi.options.OptionsBundle
import com.intellij.openapi.options.colors.AttributesDescriptor
import com.intellij.openapi.util.NlsContexts
import dejavu.intellij.language.file.DejavuBundle
import java.util.function.Supplier
import com.intellij.openapi.editor.DefaultLanguageHighlighterColors as Default

enum class HighlightColor(
    humanName: Supplier<@NlsContexts.AttributeDescriptor String>,
    default: TextAttributesKey? = null,
) {
    // 特殊关键词
    KEYWORD(DejavuBundle.messagePointer("color.settings.toml.keyword"), Default.KEYWORD),
    EXTENSION(OptionsBundle.messagePointer("options.language.defaults.metadata"), Default.METADATA),

    // 字面量
    NUMBER_HINT(DejavuBundle.messagePointer("color.settings.toml.number_hint"), Default.METADATA),
    DECIMAL(DejavuBundle.messagePointer("color.literal.decimal"), Default.NUMBER),
    INTEGER(DejavuBundle.messagePointer("color.literal.integer"), Default.NUMBER),
    STRING_HINT(DejavuBundle.messagePointer("color.settings.toml.string_hint"), Default.KEYWORD),
    STRING(OptionsBundle.messagePointer("options.language.defaults.string"), Default.STRING),
    STRING_ESCAPE(OptionsBundle.messagePointer("options.language.defaults.string"), Default.VALID_STRING_ESCAPE),
    IDENTIFIER(OptionsBundle.messagePointer("options.language.defaults.identifier"), Default.IDENTIFIER),

    //
    OPERATOR(DejavuBundle.messagePointer("color.token.operation"), Default.OPERATION_SIGN),
    OPTIONAL(DejavuBundle.messagePointer("color.token.optional"), KEYWORD.textAttributesKey),
    TAGGED(DejavuBundle.messagePointer("color.token.tagged"), OPERATOR.textAttributesKey),


    TYPE_HINT(DejavuBundle.messagePointer("color.settings.toml.type_hint"), Default.CLASS_NAME),

    CLASS_NAME(DejavuBundle.messagePointer("color.symbol.function"), Default.CLASS_NAME),
    SYM_FUNCTION(DejavuBundle.messagePointer("color.symbol.function"), Default.STATIC_METHOD),
    SYM_PROPERTY(DejavuBundle.messagePointer("color.symbol.property"), Default.STATIC_FIELD),
    SYM_VARIABLE(DejavuBundle.messagePointer("color.symbol.variable"), Default.LOCAL_VARIABLE),

    SYM_MACRO(OptionsBundle.messagePointer("options.java.attribute.descriptor.annotation.name"), Default.METADATA),
    KEY_SYMBOL(DejavuBundle.messagePointer("color.symbol.builtin"), Default.STATIC_FIELD),

    // 标点符号
    DELIMITER(DejavuBundle.messagePointer("color.delimiter"), Default.PREDEFINED_SYMBOL),
    PARENTHESES(OptionsBundle.messagePointer("options.language.defaults.parentheses"), Default.PARENTHESES),
    BRACKETS(OptionsBundle.messagePointer("options.language.defaults.brackets"), Default.BRACKETS),
    BRACES(OptionsBundle.messagePointer("options.language.defaults.braces"), Default.BRACES),
    DOT(OptionsBundle.messagePointer("options.language.defaults.dot"), Default.DOT),
    COMMA(OptionsBundle.messagePointer("options.language.defaults.comma"), Default.COMMA),
    SEMICOLON(OptionsBundle.messagePointer("options.language.defaults.semicolon"), Default.SEMICOLON),
    SET(DejavuBundle.messagePointer("color.token.set"), Default.OPERATION_SIGN),

    COMMENT_BLOCK(OptionsBundle.messagePointer("options.language.defaults.block.comment"), Default.BLOCK_COMMENT),

    // 错误
    BAD_CHARACTER(
        OptionsBundle.messagePointer("options.java.attribute.descriptor.bad.character"),
        HighlighterColors.BAD_CHARACTER
    ),
    ;


    val textAttributesKey: TextAttributesKey = TextAttributesKey.createTextAttributesKey("ygg.$name", default)
    val attributesDescriptor: AttributesDescriptor = AttributesDescriptor(humanName, textAttributesKey)
    val testSeverity: HighlightSeverity = HighlightSeverity(name, HighlightSeverity.INFORMATION.myVal)
}
