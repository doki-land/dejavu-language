package dejavu.tools

import dejavu.Dejavu
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import java.io.File

fun main(args: Array<String>) {
    if (args.isEmpty()) {
        System.err.println("Usage: dejavu parse <file> | dejavu render <file> [--from-ir] [--ctx file]")
        kotlin.system.exitProcess(1)
    }
    val json = Json { prettyPrint = true }
    when (args[0]) {
        "parse" -> {
            require(args.size >= 2)
            val ir = Dejavu.parse(File(args[1]).readText(Charsets.UTF_8))
            println(json.encodeToString(kotlinx.serialization.json.JsonObject.serializer(), ir))
        }

        "render" -> {
            require(args.size >= 2)
            val fromIr = args.contains("--from-ir")
            val ctxIdx = args.indexOf("--ctx")
            val ctx = if (ctxIdx >= 0) {
                Json.parseToJsonElement(File(args[ctxIdx + 1]).readText(Charsets.UTF_8)).jsonObject
            } else {
                kotlinx.serialization.json.buildJsonObject { }
            }
            val raw = File(args[1]).readText(Charsets.UTF_8)
            val out = if (fromIr) {
                Dejavu.render(Json.parseToJsonElement(raw).jsonObject, ctx)
            } else {
                Dejavu.renderSource(raw, ctx)
            }
            print(out)
        }

        "conformance" -> {
            val root = File(args.getOrNull(1) ?: error("conformance root required")).canonicalFile
            if (!root.isDirectory) {
                System.err.println("conformance root is not a directory: ${root.path}")
                kotlin.system.exitProcess(1)
            }
            val cases = root.listFiles()?.filter { it.isDirectory }?.sortedBy { it.name }.orEmpty()
            if (cases.isEmpty()) {
                System.err.println("no conformance cases under ${root.path}")
                kotlin.system.exitProcess(1)
            }
            var failed = 0
            cases.forEach { dir ->
                val expectedIr = Json.parseToJsonElement(
                    File(dir, "expected.ir.json").readText(Charsets.UTF_8),
                ).jsonObject
                val ctx = Json.parseToJsonElement(
                    File(dir, "context.ctx.json").readText(Charsets.UTF_8),
                ).jsonObject
                val expectedOut = File(dir, "expected.out.txt").readText(Charsets.UTF_8)
                val out = Dejavu.render(expectedIr, ctx)
                if (out != expectedOut) {
                    System.err.println("FAIL ${dir.name}: $out != $expectedOut")
                    failed++
                } else {
                    println("OK ${dir.name}")
                }
            }
            if (failed > 0) kotlin.system.exitProcess(1)
        }

        else -> {
            System.err.println("unknown command")
            kotlin.system.exitProcess(1)
        }
    }
}
