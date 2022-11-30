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
    fun render(doc: JsonObject, ctx: JsonObject): String {
        val scope = ctx.toMutableMap()
        return renderNode(doc["body"]!!, scope)
    }

    fun renderSource(source: String, ctx: JsonObject): String =
        render(T1Parser.parseToIr(source), ctx)

    private fun renderNode(node: JsonElement, scope: MutableMap<String, JsonElement>): String {
        val obj = node.jsonObject
        return when (obj["type"]!!.jsonPrimitive.content) {
            "Template" -> obj["children"]!!.jsonArray.joinToString("") { renderNode(it, scope) }
            "Text" -> obj["value"]!!.jsonPrimitive.content
            "Comment" -> ""
            "Interpolation" -> {
                val s = valueToString(eval(obj["expression"]!!, scope))
                if (obj["raw"]?.jsonPrimitive?.booleanOrNull == true) s else htmlEscape(s)
            }

            "Stmt.If" -> renderIf(obj, scope)
            "Stmt.For" -> renderFor(obj, scope)
            "Stmt.Raw" -> obj["value"]!!.jsonPrimitive.content
            "Stmt.Block" -> obj["body"]!!.jsonArray.joinToString("") { renderNode(it, scope) }
            "Stmt.Extends", "Stmt.Include", "Stmt.Super" ->
                error("extends/include/super require a template loader")

            else -> error("node not renderable: ${obj["type"]}")
        }
    }

    private fun renderIf(obj: JsonObject, scope: MutableMap<String, JsonElement>): String {
        if (truthy(eval(obj["test"]!!, scope))) {
            return obj["consequent"]!!.jsonArray.joinToString("") { renderNode(it, scope) }
        }
        for (ei in obj["elseIfs"]!!.jsonArray) {
            val e = ei.jsonObject
            if (truthy(eval(e["test"]!!, scope))) {
                return e["consequent"]!!.jsonArray.joinToString("") { renderNode(it, scope) }
            }
        }
        val alt = obj["alternate"]?.jsonArray ?: return ""
        return alt.joinToString("") { renderNode(it, scope) }
    }

    private fun renderFor(obj: JsonObject, scope: MutableMap<String, JsonElement>): String {
        val item = obj["item"]!!.jsonPrimitive.content
        val iterable = eval(obj["iterable"]!!, scope).jsonArray
        val indexName = obj["index"]?.jsonPrimitive?.content
        val out = StringBuilder()
        iterable.forEachIndexed { i, value ->
            val prev = scope[item]
            scope[item] = value
            val prevIdx = indexName?.let { scope[it] }
            if (indexName != null) scope[indexName] = JsonPrimitive(i)
            out.append(obj["body"]!!.jsonArray.joinToString("") { renderNode(it, scope) })
            if (prev == null) scope.remove(item) else scope[item] = prev
            if (indexName != null) {
                if (prevIdx == null) scope.remove(indexName) else scope[indexName] = prevIdx
            }
        }
        return out.toString()
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
