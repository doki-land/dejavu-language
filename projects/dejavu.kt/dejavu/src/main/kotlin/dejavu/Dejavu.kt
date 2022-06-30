package dejavu

import dejavu.engine.DejavuEngine
import dejavu.engine.IrRenderer
import dejavu.language.T1Parser
import dejavu.types.normalize
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject

/**
 * Public Kotlin surface for the Dejavu template engine.
 *
 * Application code should depend on the **`dejavu`** module only.
 *
 * ```kotlin
 * val out = Dejavu.renderSource("Hello, <% name %>!", buildJsonObject { put("name", "World") })
 * ```
 */
object Dejavu {
    /** Parse template source → Dejavu IR document. */
    fun parse(source: String): JsonObject = T1Parser.parseToIr(source)

    /**
     * Render IR + context → string.
     * Same IR + context must produce byte-identical output across host languages.
     */
    fun render(ir: JsonObject, ctx: JsonObject = buildJsonObject { }): String =
        IrRenderer.render(ir, ctx)

    /** Parse then render. */
    fun renderSource(source: String, ctx: JsonObject = buildJsonObject { }): String =
        IrRenderer.renderSource(source, ctx)

    /** Syntax check (parse only). */
    fun check(source: String): CheckResult =
        try {
            parse(source)
            CheckResult(valid = true, errors = emptyList())
        } catch (e: Exception) {
            CheckResult(valid = false, errors = listOf(e.message ?: e.toString()))
        }

    /** Normalize IR for semantic equality. */
    fun normalizeIr(ir: JsonObject) = normalize(ir)
}

data class CheckResult(val valid: Boolean, val errors: List<String>)

/** Instance-style alias matching other hosts. */
typealias Engine = DejavuEngine
