package dejavu.intellij.ide

import com.intellij.testFramework.fixtures.BasePlatformTestCase

class DejavuHighlightTest : BasePlatformTestCase() {
    override fun getTestDataPath(): String {
        return "src/test/testData"
    }

    fun testForLoopHighlighting() {
        myFixture.testHighlighting("highlight/for-loop.dejavu")
    }

    fun testIfConditionHighlighting() {
        myFixture.testHighlighting("highlight/if-condition.dejavu")
    }

    fun testLetBindHighlighting() {
        myFixture.testHighlighting("highlight/let-bind.dejavu")
    }

    fun testRustHighlighting() {
        myFixture.testHighlighting("highlight/rust.dejavu")
    }
}