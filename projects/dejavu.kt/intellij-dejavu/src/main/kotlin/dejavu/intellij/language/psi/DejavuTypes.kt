package dejavu.intellij.language.psi

import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import com.intellij.psi.tree.IFileElementType
import dejavu.intellij.language.DejavuLanguage

object DejavuTypes {
    val FILE = IFileElementType(DejavuLanguage)
    val TEMPLATE = DejavuElementType("TEMPLATE")
    val EXPRESSION = DejavuElementType("EXPR")
    val COMMENT = DejavuElementType("COMMENT")

    // Template types
    val IF_TEMPLATE = DejavuElementType("IF_TEMPLATE")
    val LOOP_TEMPLATE = DejavuElementType("LOOP_TEMPLATE")
    val WHILE_TEMPLATE = DejavuElementType("WHILE_TEMPLATE")
    val MATCH_TEMPLATE = DejavuElementType("MATCH_TEMPLATE")
    val BLOCK_TEMPLATE = DejavuElementType("BLOCK_TEMPLATE")
    val RAW_TEMPLATE = DejavuElementType("RAW_TEMPLATE")
    val EXPRESSION_TEMPLATE = DejavuElementType("EXPRESSION_TEMPLATE")
    val MACRO_TEMPLATE = DejavuElementType("MACRO_TEMPLATE")

    // Fragment types
    val IF_FRAGMENT = DejavuElementType("IF_FRAGMENT")
    val ELSE_IF_FRAGMENT = DejavuElementType("ELSE_IF_FRAGMENT")
    val ELSE_FRAGMENT = DejavuElementType("ELSE_FRAGMENT")
    val END_FRAGMENT = DejavuElementType("END_FRAGMENT")
    val LOOP_FRAGMENT = DejavuElementType("LOOP_FRAGMENT")
    val WHILE_FRAGMENT = DejavuElementType("WHILE_FRAGMENT")
    val UNTIL_FRAGMENT = DejavuElementType("UNTIL_FRAGMENT")
    val MATCH_FRAGMENT = DejavuElementType("MATCH_FRAGMENT")
    val CASE_FRAGMENT = DejavuElementType("CASE_FRAGMENT")
    val BLOCK_FRAGMENT = DejavuElementType("BLOCK_FRAGMENT")
    val EXTENDS_FRAGMENT = DejavuElementType("EXTENDS_FRAGMENT")
    val INCLUDE_FRAGMENT = DejavuElementType("INCLUDE_FRAGMENT")
    val RAW_FRAGMENT = DejavuElementType("RAW_FRAGMENT")
    val MACRO_FRAGMENT = DejavuElementType("MACRO_FRAGMENT")

    // Expression types
    val FUNCTION_CALL = DejavuElementType("FUNCTION_CALL")
    val ARRAY_ACCESS = DejavuElementType("ARRAY_ACCESS")
    val MEMBER_ACCESS = DejavuElementType("MEMBER_ACCESS")
    val BINARY_EXPRESSION = DejavuElementType("BINARY_EXPRESSION")
    val UNARY_EXPRESSION = DejavuElementType("UNARY_EXPRESSION")
    val PRIMARY_EXPRESSION = DejavuElementType("PRIMARY_EXPRESSION")

    /** 所有的起始标记，包括 `<%`、`<%_`、`<%-`、`<%~`、`<%=`。 */
    val SLOT_L = DejavuTokenType("<%")

    /** 所有的结束标记，包括 `%>`、`_%>`、`-%>`、`~%>`、`=%>`。 */
    val SLOT_R = DejavuTokenType("%>")
    val COMMENT_L = DejavuTokenType("<#")
    val COMMENT_R = DejavuTokenType("#>")
    val TEXT = DejavuTokenType("TEXT")

    val COMMENT_CONTENT = DejavuTokenType("COMMENT_CONTENT")

    // Tokens - Keywords
    val KEYWORD_IF = DejavuTokenType("if")
    val KEYWORD_END = DejavuTokenType("end")
    val KEYWORD_LOOP = DejavuTokenType("loop")
    val KEYWORD_MATCH = DejavuTokenType("match")
    val KEYWORD_CASE = DejavuTokenType("case")
    val KEYWORD_ELSE = DejavuTokenType("else")
    val KEYWORD_WHILE = DejavuTokenType("while")
    val KEYWORD_UNTIL = DejavuTokenType("until")
    val KEYWORD_EXTENDS = DejavuTokenType("extends")
    val KEYWORD_BLOCK = DejavuTokenType("block")
    val KEYWORD_INCLUDE = DejavuTokenType("include")
    val KEYWORD_RAW = DejavuTokenType("raw")
    val KEYWORD_SUPER = DejavuTokenType("super")
    val KEYWORD_LOOPER = DejavuTokenType("looper")
    val KEYWORD_LET = DejavuTokenType("let")
    val KEYWORD_IN = DejavuTokenType("in")
    val KEYWORD_MACRO = DejavuTokenType("macro")

    // Tokens - Program
    val IDENTIFIER = DejavuTokenType("IDENTIFIER")
    val NUMBER = DejavuTokenType("NUMBER")
    val STRING = DejavuTokenType("STRING")
    val PUNCTUATION = DejavuTokenType("PUNCTUATION")

    // Arithmetic operators
    val PLUS = DejavuTokenType("+")
    val MINUS = DejavuTokenType("-")
    val MULTIPLY = DejavuTokenType("*")
    val DIVIDE = DejavuTokenType("/")
    val MODULO = DejavuTokenType("%")

    // Comparison operators
    val EQUAL = DejavuTokenType("==")
    val NOT_EQUAL = DejavuTokenType("!=")
    val LESS_THAN = DejavuTokenType("<")
    val GREATER_THAN = DejavuTokenType(">")
    val LESS_THAN_OR_EQUAL = DejavuTokenType("<=")
    val GREATER_THAN_OR_EQUAL = DejavuTokenType(">=")

    // Logical operators
    val AND = DejavuTokenType("&&")
    val OR = DejavuTokenType("||")
    val NOT = DejavuTokenType("!")

    // Assignment operators
    val ASSIGN = DejavuTokenType("=")
    val PLUS_ASSIGN = DejavuTokenType("+=")
    val MINUS_ASSIGN = DejavuTokenType("-=")
    val MULTIPLY_ASSIGN = DejavuTokenType("*=")
    val DIVIDE_ASSIGN = DejavuTokenType("/=")
    val MODULO_ASSIGN = DejavuTokenType("%=")

    // Brackets for function calls and array access
    val LPAREN = DejavuTokenType("(")
    val RPAREN = DejavuTokenType(")")
    val LBRACKET = DejavuTokenType("[")
    val RBRACKET = DejavuTokenType("]")
    val DOT = DejavuTokenType(".")
    val COMMA = DejavuTokenType(",")

    val WHITESPACE: IElementType = TokenType.WHITE_SPACE
}
