package dejavu.intellij.language

import com.intellij.lexer.Lexer
import com.intellij.testFramework.LexerTestCase
import dejavu.intellij.language.parser.DejavuLexer

class DokiLexerTest : LexerTestCase() {
    override fun createLexer(): Lexer {
        return DejavuLexer(DokiLanguage.LanguageConfig)
    }

    override fun getDirPath(): String {
        return "lexer"
    }

    override fun getPathToTestDataFile(extension: String): String {
        return "src/test/testData/lexer/${getTestName(false)}$extension"
    }

    fun testEmpty() {
        doTest("")
    }

    fun testWhitespace() {
        doTest("   \n\t  ")
    }

    fun testComments() {
        doTest("{# This is a comment #}")
    }

    fun testExpressions() {
        doTest("{%= 1 + 2 %}")
    }

    fun testIfStatements() {
        doTest("{% if (true) %}Hello{% end %}")
    }

    fun testText() {
        doTest("Hello World")
    }

    fun testBasic() {
        doTest("{%= name %} is {%= age %} years old")
    }

    fun testDejavuBasic() {
        doTest("{%~ if user ~%}Welcome {%= user.name %}{%~ end ~%}")
    }

    fun testMatchStatements() {
        doTest("{% match value %}{% case 1 %}One{% case 2 %}Two{% end %}")
    }

    fun testStatements() {
        doTest("{% loop items as item %}{%= item %}{% end %}")
    }

    fun testWhitespaceControl() {
        doTest("{%_ if true _%}Hello{%- end -%}")
    }
}