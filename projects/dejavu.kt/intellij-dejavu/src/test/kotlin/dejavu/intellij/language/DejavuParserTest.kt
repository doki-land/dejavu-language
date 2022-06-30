package dejavu.intellij.language

import com.intellij.testFramework.ParsingTestCase
import dejavu.intellij.language.parser.DejavuParserDefinition

class DejavuParserTest : ParsingTestCase("parser", "dejavu", DejavuParserDefinition()) {
    override fun getTestDataPath(): String {
        return "src/test/testData"
    }

    fun testEmpty() {
        doTest(true, true)
    }

    fun testExpressions() {
        doTest(true, true)
    }

    fun testIfStatements() {
        doTest(true, true)
    }

    fun testComments() {
        doTest(true, true)
    }

    fun testText() {
        doTest(true, true)
    }

    fun testSlot() {
        doTest(true, true)
    }

    fun testMatchStatements() {
        doTest(true, true)
    }

    fun testLoopStatements() {
        doTest(true, true)
    }

    fun testIfElseStatements() {
        doTest(true, false)
    }

    fun testWhileStatements() {
        doTest(true, true)
    }

    fun testWhileElseStatements() {
        doTest(true, true)
    }

    fun testUntilStatements() {
        doTest(true, true)
    }

    fun testUntilElseStatements() {
        doTest(true, true)
    }

    fun testLetStatements() {
        doTest(true, true)
    }

    fun testMacroStatements() {
        doTest(true, true)
    }

    fun testBlockStatements() {
        doTest(true, true)
    }

    fun testRawStatements() {
        doTest(true, true)
    }

    fun testIncludeStatements() {
        doTest(true, true)
    }

    fun testUsingStatements() {
        doTest(true, true)
    }

    fun testExtendsStatements() {
        doTest(true, true)
    }

    fun testFunctionCallArrayAccess() {
        doTest(true, true)
    }

    override fun includeRanges(): Boolean {
        return true
    }
}
