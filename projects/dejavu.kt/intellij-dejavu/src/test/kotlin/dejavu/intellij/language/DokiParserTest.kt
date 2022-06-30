package dejavu.intellij.language

import com.intellij.testFramework.ParsingTestCase
import dejavu.intellij.language.parser.DokiParserDefinition

class DokiParserTest : ParsingTestCase("parser", "doki", DokiParserDefinition()) {
    override fun getTestDataPath(): String {
        return "src/test/testData"
    }

    fun testComments() {
        doTest(true, true)
    }

    override fun includeRanges(): Boolean {
        return true
    }
}
