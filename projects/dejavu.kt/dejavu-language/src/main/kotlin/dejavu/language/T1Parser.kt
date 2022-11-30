package dejavu.language

import dejavu.types.DEFAULT_LANGUAGE
import kotlinx.serialization.json.*

/** Parse T1 template source into Dejavu IR. */
object T1Parser {
    fun parseToIr(source: String, file: String = "template.dejavu"): JsonObject = buildJsonObject {
        put("irVersion", "1.0")
        put("language", DEFAULT_LANGUAGE)
        put("body", buildJsonObject {
            put("type", "Template")
            put("children", JsonArray(TemplateParser(source, file).parse()))
        })
    }
}

private class TemplateParser(
    private val source: String,
    private val file: String,
) {
    fun parse(): List<JsonElement> {
        val (children, _) = parseBody(0, emptySet())
        return children
    }

    private fun parseBody(i0: Int, stop: Set<CodeHead.Kind>): Pair<List<JsonElement>, Int> {
        val children = mutableListOf<JsonElement>()
        var i = i0

        while (i < source.length) {
            if (source.startsWith("<%", i) && stop.isNotEmpty()) {
                val block = readCodeBlock(i)
                val head = classifyCode(block.content, block.contentBase, block.tokens)
                if (head.kind in stop) return children to i
            }

            if (source.startsWith("<#", i)) {
                val end = findDelimiter(i + 2, "#>")
                if (end < 0) {
                    throw ParseError(
                        message = "unclosed comment",
                        file = file,
                        start = i,
                        length = 2,
                        label = "comment starts here",
                    )
                }
                children += buildJsonObject {
                    put("type", "Comment")
                    put("value", source.substring(i + 2, end))
                }
                i = end + 2
                continue
            }

            if (source.startsWith("<%!", i)) {
                children += buildJsonObject {
                    put("type", "Text")
                    put("value", "<%")
                }
                i += 3
                continue
            }

            if (source.startsWith("<%", i)) {
                val open = i
                val block = readCodeBlock(i)
                i = block.next
                val head = classifyCode(block.content, block.contentBase, block.tokens)

                if (stop.isNotEmpty() && head.kind in stop) {
                    return children to open
                }

                when (head) {
                    is CodeHead.If -> {
                        val (node, ni) = parseIf(i, head, block.trim)
                        children += node
                        i = ni
                    }

                    is CodeHead.Loop -> {
                        val (node, ni) = parseLoop(i, head, block.trim)
                        children += node
                        i = ni
                    }

                    CodeHead.EndIf, CodeHead.EndLoop, CodeHead.Else, is CodeHead.ElseIf -> {
                        if (stop.isEmpty()) {
                            throw ParseError(
                                message = "unexpected control `${head.kind}`",
                                file = file,
                                start = block.contentBase,
                                length = 1,
                            )
                        }
                        return children to open
                    }

                    is CodeHead.Expr -> {
                        children += buildJsonObject {
                            put("type", "Interpolation")
                            put(
                                "expression",
                                parseExpr(head.exprSlice, source = source, file = file, base = head.exprAbs),
                            )
                            put("trim", block.trim)
                        }
                    }
                }
                continue
            }

            val next = nextMarkup(i)
            if (next == null) {
                children += buildJsonObject {
                    put("type", "Text")
                    put("value", source.substring(i))
                }
                break
            }
            if (next > i) {
                children += buildJsonObject {
                    put("type", "Text")
                    put("value", source.substring(i, next))
                }
                i = next
            } else {
                i += 1
            }
        }

        return children to i
    }

    private fun parseIf(i0: Int, head: CodeHead.If, trim: String): Pair<JsonObject, Int> {
        var i = i0
        val test = parseExpr(head.testSlice, source = source, file = file, base = head.testAbs)
        var (consequent, pos) = parseBody(i, setOf(CodeHead.Kind.ElseIf, CodeHead.Kind.Else, CodeHead.Kind.EndIf))
        i = pos
        val elseIfs = mutableListOf<JsonElement>()
        var alternate: List<JsonElement>? = null

        while (true) {
            val block = readCodeBlock(i)
            when (val h = classifyCode(block.content, block.contentBase, block.tokens)) {
                is CodeHead.ElseIf -> {
                    i = block.next
                    val t = parseExpr(h.testSlice, source = source, file = file, base = h.testAbs)
                    val (body, n) = parseBody(i, setOf(CodeHead.Kind.ElseIf, CodeHead.Kind.Else, CodeHead.Kind.EndIf))
                    elseIfs += buildJsonObject {
                        put("type", "Stmt.ElseIf")
                        put("test", t)
                        put("consequent", JsonArray(body))
                        put("trim", "none")
                    }
                    i = n
                }

                CodeHead.Else -> {
                    i = block.next
                    val (body, n) = parseBody(i, setOf(CodeHead.Kind.EndIf))
                    alternate = body
                    i = n
                    val end = readCodeBlock(i)
                    if (classifyCode(end.content, end.contentBase, end.tokens) !is CodeHead.EndIf) {
                        throw ParseError(
                            message = "expected `end if`",
                            file = file,
                            start = end.contentBase,
                            length = 1,
                        )
                    }
                    i = end.next
                    break
                }

                CodeHead.EndIf -> {
                    i = block.next
                    break
                }

                else -> throw ParseError(
                    message = "expected if closer, got `${h.kind}`",
                    file = file,
                    start = block.contentBase,
                    length = 1,
                )
            }
        }

        return buildJsonObject {
            put("type", "Stmt.If")
            put("test", test)
            put("consequent", JsonArray(consequent))
            put("elseIfs", JsonArray(elseIfs))
            if (alternate != null) put("alternate", JsonArray(alternate))
            put("trim", trim)
        } to i
    }

    private fun parseLoop(i0: Int, head: CodeHead.Loop, trim: String): Pair<JsonObject, Int> {
        var i = i0
        val iterable = parseExpr(head.iterSlice, source = source, file = file, base = head.iterAbs)
        val (body, pos) = parseBody(i, setOf(CodeHead.Kind.EndLoop))
        i = pos
        val end = readCodeBlock(i)
        if (classifyCode(end.content, end.contentBase, end.tokens) !is CodeHead.EndLoop) {
            throw ParseError(
                message = "expected `end loop`",
                file = file,
                start = end.contentBase,
                length = 1,
            )
        }
        return buildJsonObject {
            put("type", "Stmt.For")
            put("item", head.item)
            put("iterable", iterable)
            put("body", JsonArray(body))
            put("trim", trim)
        } to end.next
    }

    private data class CodeBlock(
        val tokens: List<CodeToken>,
        val content: String,
        val contentBase: Int,
        val trim: String,
        val next: Int,
    )

    private fun readCodeBlock(i: Int): CodeBlock {
        if (!source.startsWith("<%", i)) {
            throw ParseError(
                message = "expected code open `<%`",
                file = file,
                start = i,
                length = 1,
            )
        }
        var j = i + 2
        var trim = "none"
        val mod = source.getOrNull(j)
        if (mod == '.' || mod == '_' || mod == '-' || mod == '~' || mod == '=') {
            trim = when (mod) {
                '_' -> "ws"
                '-' -> "nl"
                '~' -> "ws_nl"
                '=' -> "all"
                else -> "none"
            }
            j++
        }
        val end = findDelimiter(j, "%>")
        if (end < 0) {
            throw ParseError(
                message = "unclosed code block",
                file = file,
                start = i,
                length = 2,
                label = "opens here",
            )
        }
        var contentBase = j
        while (contentBase < end) {
            val c = source[contentBase].code
            if (c == 0x20 || c == 0x09 || c == 0x0a || c == 0x0d) contentBase++
            else break
        }
        var contentEnd = end
        while (contentEnd > contentBase) {
            val c = source[contentEnd - 1].code
            if (c == 0x20 || c == 0x09 || c == 0x0a || c == 0x0d) contentEnd--
            else break
        }
        val content = source.substring(contentBase, contentEnd)
        val tokens = CodeLexer.lexCode(content, source = source, file = file, base = contentBase)
            .filter { it.kind != CodeTokenKind.CodeEnd }
        return CodeBlock(tokens = tokens, content = content, contentBase = contentBase, trim = trim, next = end + 2)
    }

    /** Scan for a delimiter without using regex. */
    private fun findDelimiter(from: Int, delim: String): Int {
        val d0 = delim[0].code
        var i = from
        while (i + delim.length <= source.length) {
            if (source[i].code != d0) {
                i++
                continue
            }
            var ok = true
            for (k in 1 until delim.length) {
                if (source[i + k].code != delim[k].code) {
                    ok = false
                    break
                }
            }
            if (ok) return i
            i++
        }
        return -1
    }

    private fun nextMarkup(from: Int): Int? {
        var i = from
        while (i + 1 < source.length) {
            if (source[i].code != 0x3c /* < */) {
                i++
                continue
            }
            val n = source[i + 1].code
            if (n == 0x25 /* % */ || n == 0x23 /* # */) return i
            i++
        }
        return null
    }
}
