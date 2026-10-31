package dejavu.engine

import dejavu.Dejavu
import dejavu.types.normalize
import kotlinx.serialization.json.*
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ConformanceTest {
    private val root: File = findConformanceRoot()

    @Test
    fun irSuitesRenderFromExpectedIr() {
        val suites = listOf("loop-metadata", "control-flow")
        val conformance = requireNotNull(root.parentFile)
        val json = Json { ignoreUnknownKeys = true }
        suites.forEach { name ->
            val suite = File(conformance, name)
            assertTrue(suite.isDirectory, "missing $suite")
            suite.listFiles()!!.filter { it.isDirectory }.sortedBy { it.name }.forEach { dir ->
                val expectedIr = json.parseToJsonElement(
                    File(dir, "expected.ir.json").readText(Charsets.UTF_8),
                ).jsonObject
                val ctx = json.parseToJsonElement(
                    File(dir, "context.ctx.json").readText(Charsets.UTF_8),
                ).jsonObject
                val expectedOut = File(dir, "expected.out.txt").readText(Charsets.UTF_8).trimEnd()
                assertEquals(expectedOut, Dejavu.render(expectedIr, ctx).trimEnd(), "render mismatch in $name/${dir.name}")
            }
        }
    }

    @Test
    fun t1CasesParseAndRenderFromExpectedIr() {
        assertTrue(root.isDirectory, "missing $root")
        val json = Json { ignoreUnknownKeys = true }
        root.listFiles()!!.filter { it.isDirectory }.sortedBy { it.name }.forEach { dir ->
            val input = File(dir, "input.dejavu").readText()
            val expectedIrText = File(dir, "expected.ir.json").readText()
            val expectedIr = json.parseToJsonElement(expectedIrText).jsonObject
            val ctx = json.parseToJsonElement(File(dir, "context.ctx.json").readText()).jsonObject
            val expectedOut = File(dir, "expected.out.txt").readText()

            val got = Dejavu.parse(input)
            assertEquals(normalize(expectedIr), normalize(got), "IR mismatch in ${dir.name}")

            val out = Dejavu.render(expectedIr, ctx)
            assertEquals(expectedOut, out, "render mismatch in ${dir.name}")
        }
    }

    companion object {
        private val REL = "specifications/conformance/t1"

        private fun findConformanceRoot(): File {
            val markers = listOf(
                File("../../../$REL"),
                File("../../../../$REL"),
                File(REL),
            )
            for (c in markers) {
                val f = c.canonicalFile
                if (f.isDirectory) return f
            }
            var dir = File(System.getProperty("user.dir")).canonicalFile
            repeat(10) {
                val candidate = File(dir, REL)
                if (candidate.isDirectory) return candidate
                dir = dir.parentFile ?: return@repeat
            }
            return File("../../../$REL").canonicalFile
        }
    }
}
