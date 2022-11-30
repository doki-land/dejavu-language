package dejavu.types

import kotlinx.serialization.json.*

val DEFAULT_LANGUAGE: JsonObject = buildJsonObject {
    put("syntaxMode", "template")
    putJsonObject("template") {
        put("codeStart", "<%")
        put("codeEnd", "%>")
        put("commentStart", "<#")
        put("commentEnd", "#>")
        put("supportFilterPipe", true)
        put("legacyFor", false)
    }
}

fun normalize(node: JsonElement?): JsonElement? {
    if (node == null || node is JsonNull) return null
    if (node is JsonArray) {
        return JsonArray(node.mapNotNull { normalize(it) })
    }
    if (node is JsonObject) {
        val type = node["type"]?.jsonPrimitive?.content
        if (type == "Text" && node["value"]?.jsonPrimitive?.content == "") return null
        val out = buildJsonObject {
            node.keys.sorted().forEach { key ->
                if (key == "span") return@forEach
                if (key == "raw" && node[key]?.jsonPrimitive?.booleanOrNull == false) return@forEach
                normalize(node[key])?.let { put(key, it) }
            }
        }
        return out
    }
    return node
}

fun valueToString(v: JsonElement?): String = when (v) {
    null, is JsonNull -> ""
    is JsonPrimitive -> when {
        v.isString -> v.content
        v.booleanOrNull != null -> v.boolean.toString()
        else -> v.content
    }

    else -> v.toString()
}

fun applyFilter(name: String, value: JsonElement?, args: List<JsonElement?>): JsonElement {
    return when (name) {
        "uppercase" -> JsonPrimitive(valueToString(value).uppercase())
        "lowercase" -> JsonPrimitive(valueToString(value).lowercase())
        "trim" -> JsonPrimitive(valueToString(value).trim())
        "default" -> if (value == null || value is JsonNull || (value is JsonPrimitive && value.isString && value.content.isEmpty()))
            args.firstOrNull() ?: JsonNull else value

        "length" -> JsonPrimitive(
            when (value) {
                is JsonArray -> value.size
                is JsonObject -> value.size
                is JsonPrimitive -> if (value.isString) value.content.length else 0
                else -> 0
            }
        )

        "join" -> {
            val sep = args.firstOrNull()?.let { valueToString(it) } ?: ","
            if (value is JsonArray) JsonPrimitive(value.joinToString(sep) { valueToString(it) })
            else JsonPrimitive(valueToString(value))
        }

        "replace" -> {
            val from = valueToString(args.getOrNull(0))
            val to = valueToString(args.getOrNull(1))
            JsonPrimitive(valueToString(value).replace(from, to))
        }

        else -> error("unknown filter `$name`")
    }
}
