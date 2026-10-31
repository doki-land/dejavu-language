package dejavu.engine

import dejavu.language.T1Parser
import dejavu.types.applyFilter
import dejavu.types.valueToString
import kotlinx.serialization.json.*

/**
 * Canonical IR evaluator. Same IR + Context must yield identical output
 * across host languages (byte-for-byte for T1 cases).
 */
object IrRenderer {
    /** Pending `break` / `continue` signal, shared through one render. */
    private class ControlState {
        var signal: String? = null
    }

    private const val FORLOOP_NAME = "forloop"

    fun render(doc: JsonObject, ctx: JsonObject): String {
        val scope = ctx.toMutableMap()
        val control = ControlState()
        val out = renderNode(doc["body"]!!, scope, control)
        // The template top level is a lexical unit: an escaping signal fails
        // the render instead of binding to an outer loop.
        if (control.signal != null) {
            error("dejavu.template.control-outside-loop: ${control.signal}")
        }
        return out
    }

    fun renderSource(source: String, ctx: JsonObject): String =
        render(T1Parser.parseToIr(source), ctx)

    /** Render a sibling list, stopping after the node that set a control signal. */
    private fun renderBody(
        nodes: JsonArray,
        scope: MutableMap<String, JsonElement>,
        control: ControlState,
    ): String {
        val out = StringBuilder()
        for (node in nodes) {
            out.append(renderNode(node, scope, control))
            // A pending signal stops the rest of the list; the enclosing loop
            // (or lexical-unit boundary) consumes it.
            if (control.signal != null) break
        }
        return out.toString()
    }

    private fun setControl(control: ControlState, signal: String): String {
        control.signal = signal
        return ""
    }

    private fun renderNode(
        node: JsonElement,
        scope: MutableMap<String, JsonElement>,
        control: ControlState,
    ): String {
        val obj = node.jsonObject
        return when (obj["type"]!!.jsonPrimitive.content) {
            "Template" -> renderBody(obj["children"]!!.jsonArray, scope, control)
            "Text" -> obj["value"]!!.jsonPrimitive.content
            "Comment" -> ""
            "Interpolation" -> {
                val s = valueToString(eval(obj["expression"]!!, scope))
                if (obj["raw"]?.jsonPrimitive?.booleanOrNull == true) s else htmlEscape(s)
            }

            "Stmt.If" -> renderIf(obj, scope, control)
            "Stmt.Break" -> setControl(control, "break")
            "Stmt.Continue" -> setControl(control, "continue")
            "Stmt.For" -> renderFor(obj, scope, control)
            "Stmt.Raw" -> obj["value"]!!.jsonPrimitive.content
            "Stmt.Block" -> {
                val out = renderBody(obj["body"]!!.jsonArray, scope, control)
                // A block body is its own lexical unit.
                if (control.signal != null) {
                    error("dejavu.template.control-outside-loop: ${control.signal}")
                }
                out
            }

            "Stmt.Extends", "Stmt.Include", "Stmt.Super" ->
                error("extends/include/super require a template loader")

            else -> error("node not renderable: ${obj["type"]}")
        }
    }

    private fun renderIf(
        obj: JsonObject,
        scope: MutableMap<String, JsonElement>,
        control: ControlState,
    ): String {
        if (truthy(eval(obj["test"]!!, scope))) {
            return renderBody(obj["consequent"]!!.jsonArray, scope, control)
        }
        for (ei in obj["elseIfs"]!!.jsonArray) {
            val e = ei.jsonObject
            if (truthy(eval(e["test"]!!, scope))) {
                return renderBody(e["consequent"]!!.jsonArray, scope, control)
            }
        }
        val alt = obj["alternate"]?.jsonArray ?: return ""
        return renderBody(alt, scope, control)
    }

    private fun renderFor(
        obj: JsonObject,
        scope: MutableMap<String, JsonElement>,
        control: ControlState,
    ): String {
        val item = obj["item"]!!.jsonPrimitive.content
        val iterable = eval(obj["iterable"]!!, scope).jsonArray
        val length = iterable.size
        // 循环元数据：每次迭代在 body 作用域隐式绑定 `forloop`。
        // depth 入口规则：循环入口时可见的 forloop 是对象且 depth 为 >=1 的
        // 整数则 +1，否则从 1 重启（与 Rust/TS 宿主一致）。
        val prevMeta = scope[FORLOOP_NAME]
        val hadPrevMeta = scope.containsKey(FORLOOP_NAME)
        val depth = forloopDepth(prevMeta)
        val out = StringBuilder()
        for (i in 0 until length) {
            // 绑定恢复由键是否存在决定：键存在但值为 null 与键缺失不同。
            val prevItem = scope[item]
            val hadPrevItem = scope.containsKey(item)
            scope[item] = iterable[i]
            scope[FORLOOP_NAME] = forloopMeta(i, length, depth)
            out.append(renderBody(obj["body"]!!.jsonArray, scope, control))
            // The bound loop consumes the signal once, after the iteration's
            // bindings are restored (normal loop exit).
            val signal = control.signal
            control.signal = null
            restore(scope, FORLOOP_NAME, hadPrevMeta, prevMeta)
            restore(scope, item, hadPrevItem, prevItem)
            if (signal == "break") break
            if (signal == "continue") continue
        }
        return out.toString()
    }

    /** One iteration's loop metadata, mirroring the Rust `forloop_meta`. */
    private fun forloopMeta(i: Int, length: Int, depth: Long): JsonObject = buildJsonObject {
        put("index", i + 1)
        put("index0", i)
        put("first", i == 0)
        put("last", i + 1 == length)
        put("length", length)
        put("depth", depth)
    }

    /** `depth` entry rule: only an object with an integer `depth >= 1` increments. */
    private fun forloopDepth(prev: JsonElement?): Long {
        val depth = ((prev as? JsonObject)?.get("depth") as? JsonPrimitive)?.longOrNull
        return if (depth != null && depth >= 1) depth + 1 else 1
    }

    private fun restore(
        scope: MutableMap<String, JsonElement>,
        key: String,
        hadPrev: Boolean,
        prev: JsonElement?,
    ) {
        if (hadPrev) scope[key] = prev!! else scope.remove(key)
    }

    private fun eval(expr: JsonElement, scope: Map<String, JsonElement>): JsonElement {
        val obj = expr.jsonObject
        return when (obj["type"]!!.jsonPrimitive.content) {
            "Expr.Literal" -> obj["value"] ?: JsonNull
            "Expr.Identifier" -> scope[obj["name"]!!.jsonPrimitive.content] ?: JsonNull
            "Expr.Member" -> {
                val o = eval(obj["object"]!!, scope)
                if (o is JsonObject) o[obj["property"]!!.jsonPrimitive.content] ?: JsonNull else JsonNull
            }

            "Expr.Index" -> {
                val o = eval(obj["object"]!!, scope)
                val idx = eval(obj["index"]!!, scope)
                when {
                    o is JsonArray && idx is JsonPrimitive && idx.intOrNull != null ->
                        o.getOrNull(idx.int) ?: JsonNull

                    o is JsonObject && idx is JsonPrimitive && idx.isString ->
                        o[idx.content] ?: JsonNull

                    else -> JsonNull
                }
            }

            "Expr.Binary" -> evalBinary(
                obj["operator"]!!.jsonPrimitive.content,
                eval(obj["left"]!!, scope),
                eval(obj["right"]!!, scope),
            )

            "Expr.Unary" -> {
                val v = eval(obj["argument"]!!, scope)
                when (obj["operator"]!!.jsonPrimitive.content) {
                    "!" -> JsonPrimitive(!truthy(v))
                    "-" -> {
                        val n = (v as? JsonPrimitive)?.doubleOrNull
                        if (n != null) JsonPrimitive(-n) else JsonNull
                    }

                    "+" -> v
                    else -> JsonNull
                }
            }

            "Expr.Pipe" -> {
                val value = eval(obj["expression"]!!, scope)
                val args = obj["arguments"]!!.jsonArray.map { eval(it, scope) }
                applyFilter(obj["filter"]!!.jsonPrimitive.content, value, args)
            }

            "Expr.Call" -> error("calls not supported in T1 eval")
            else -> error("invalid expression: ${obj["type"]}")
        }
    }

    private fun evalBinary(op: String, l: JsonElement, r: JsonElement): JsonElement {
        val ld = (l as? JsonPrimitive)?.doubleOrNull
        val rd = (r as? JsonPrimitive)?.doubleOrNull
        return when (op) {
            "+" -> when {
                ld != null && rd != null -> num(ld + rd)
                else -> JsonPrimitive(valueToString(l) + valueToString(r))
            }

            "-" -> if (ld != null && rd != null) num(ld - rd) else JsonNull
            "*" -> if (ld != null && rd != null) num(ld * rd) else JsonNull
            "/" -> if (ld != null && rd != null) num(ld / rd) else JsonNull
            "%" -> if (ld != null && rd != null) num(ld % rd) else JsonNull
            "==" -> JsonPrimitive(l == r)
            "!=" -> JsonPrimitive(l != r)
            "<" -> JsonPrimitive(ld != null && rd != null && ld < rd)
            "<=" -> JsonPrimitive(ld != null && rd != null && ld <= rd)
            ">" -> JsonPrimitive(ld != null && rd != null && ld > rd)
            ">=" -> JsonPrimitive(ld != null && rd != null && ld >= rd)
            "&&" -> JsonPrimitive(truthy(l) && truthy(r))
            "||" -> JsonPrimitive(truthy(l) || truthy(r))
            "in" -> JsonPrimitive(
                when (r) {
                    is JsonArray -> r.any { it == l }
                    is JsonPrimitive -> if (r.isString) r.content.contains(valueToString(l)) else false
                    else -> false
                }
            )

            else -> JsonNull
        }
    }

    private fun num(v: Double): JsonPrimitive =
        if (v % 1.0 == 0.0) JsonPrimitive(v.toLong()) else JsonPrimitive(v)

    private fun truthy(v: JsonElement?): Boolean = when (v) {
        null, JsonNull -> false
        is JsonPrimitive -> when {
            v.booleanOrNull != null -> v.boolean
            v.doubleOrNull != null -> v.double != 0.0
            v.isString -> v.content.isNotEmpty()
            else -> true
        }

        is JsonArray -> v.isNotEmpty()
        is JsonObject -> v.isNotEmpty()
        else -> true
    }

    private fun htmlEscape(s: String): String =
        s.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#39;")
}

class DejavuEngine {
    fun parse(source: String): JsonObject = T1Parser.parseToIr(source)
    fun render(ir: JsonObject, ctx: JsonObject): String = IrRenderer.render(ir, ctx)
    fun renderSource(source: String, ctx: JsonObject): String = IrRenderer.renderSource(source, ctx)
}
