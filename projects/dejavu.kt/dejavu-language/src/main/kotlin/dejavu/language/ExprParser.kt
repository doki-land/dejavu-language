package dejavu.language

import kotlinx.serialization.json.*

/** Pratt-style expression parser over a [CodeToken] stream. */
class ExprParser(
    private val source: String,
    private val file: String,
    private val base: Int,
    input: String,
) {
    private val tokens: List<CodeToken> = CodeLexer.lexCode(input, source = source, file = file, base = base)
    private var pos = 0

    init {
        for (t in tokens) {
            if (t.kind == CodeTokenKind.CodeEnd) {
                throw ParseError(
                    message = "unexpected `%>` inside expression",
                    file = file,
                    start = base + t.start,
                    length = t.end - t.start,
                )
            }
        }
    }

    fun parse(): JsonObject {
        val expr = parsePipe()
        if (pos != tokens.size) {
            val span = peekSpan()
            throw ParseError(
                message = "trailing input in expression",
                file = file,
                start = span.first,
                length = span.second,
                label = "unexpected",
            )
        }
        return expr
    }

    private fun peek(): CodeToken? = tokens.getOrNull(pos)

    private fun peekKind(): CodeTokenKind? = tokens.getOrNull(pos)?.kind

    private fun peekSpan(): Pair<Int, Int> {
        val t = tokens.getOrNull(pos)
        if (t != null) return (base + t.start) to maxOf(1, t.end - t.start)
        val last = tokens.lastOrNull()
        val end = base + (last?.end ?: 0)
        return end to 1
    }

    private fun bump(): CodeToken? = tokens.getOrNull(pos++)

    private fun expectIdent(): String {
        val t = bump()
        if (t?.kind == CodeTokenKind.Ident) return t.text
        val span = peekSpan()
        throw ParseError(
            message = "expected identifier",
            file = file,
            start = span.first,
            length = span.second,
            label = "expected ident",
        )
    }

    private fun parsePipe(): JsonObject {
        var left = parseOr()
        while (peekKind() == CodeTokenKind.PipeOp) {
            bump()
            val filter = expectIdent()
            val args = mutableListOf<JsonElement>()
            if (peekKind() == CodeTokenKind.LParen) {
                bump()
                if (peekKind() != CodeTokenKind.RParen) {
                    while (true) {
                        args += parsePipe()
                        if (peekKind() == CodeTokenKind.Comma) {
                            bump()
                            continue
                        }
                        break
                    }
                }
                if (bump()?.kind != CodeTokenKind.RParen) {
                    val span = peekSpan()
                    throw ParseError(
                        message = "expected `)` after filter arguments",
                        file = file,
                        start = span.first,
                        length = span.second,
                    )
                }
            }
            left = buildJsonObject {
                put("type", "Expr.Pipe")
                put("expression", left)
                put("filter", filter)
                put("arguments", JsonArray(args))
            }
        }
        return left
    }

    private fun parseOr(): JsonObject {
        var left = parseAnd()
        while (peekKind() == CodeTokenKind.OrOr) {
            bump()
            left = bin("||", left, parseAnd())
        }
        return left
    }

    private fun parseAnd(): JsonObject {
        var left = parseCmp()
        while (peekKind() == CodeTokenKind.AndAnd) {
            bump()
            left = bin("&&", left, parseCmp())
        }
        return left
    }

    private fun parseCmp(): JsonObject {
        val left = parseAdd()
        val op = when (peekKind()) {
            CodeTokenKind.EqEq -> "=="
            CodeTokenKind.NotEq -> "!="
            CodeTokenKind.LessEq -> "<="
            CodeTokenKind.GreaterEq -> ">="
            CodeTokenKind.Less -> "<"
            CodeTokenKind.Greater -> ">"
            CodeTokenKind.In -> "in"
            else -> null
        }
        if (op != null) {
            bump()
            return bin(op, left, parseAdd())
        }
        return left
    }

    private fun parseAdd(): JsonObject {
        var left = parseMul()
        while (true) {
            val kind = peekKind()
            if (kind == CodeTokenKind.Plus || kind == CodeTokenKind.Minus) {
                val op = if (kind == CodeTokenKind.Plus) "+" else "-"
                bump()
                left = bin(op, left, parseMul())
            } else break
        }
        return left
    }

    private fun parseMul(): JsonObject {
        var left = parseUnary()
        while (true) {
            val kind = peekKind()
            if (kind == CodeTokenKind.Star || kind == CodeTokenKind.Slash || kind == CodeTokenKind.Percent) {
                val op = when (kind) {
                    CodeTokenKind.Star -> "*"
                    CodeTokenKind.Slash -> "/"
                    else -> "%"
                }
                bump()
                left = bin(op, left, parseUnary())
            } else break
        }
        return left
    }

    private fun parseUnary(): JsonObject {
        val kind = peekKind()
        if (kind == CodeTokenKind.Bang || kind == CodeTokenKind.Minus || kind == CodeTokenKind.Plus) {
            val op = when (kind) {
                CodeTokenKind.Bang -> "!"
                CodeTokenKind.Minus -> "-"
                else -> "+"
            }
            bump()
            return buildJsonObject {
                put("type", "Expr.Unary")
                put("operator", op)
                put("argument", parseUnary())
            }
        }
        return parsePostfix()
    }

    private fun parsePostfix(): JsonObject {
        var left = parsePrimary()
        while (true) {
            when (peekKind()) {
                CodeTokenKind.Dot -> {
                    bump()
                    left = buildJsonObject {
                        put("type", "Expr.Member")
                        put("object", left)
                        put("property", expectIdent())
                    }
                }

                CodeTokenKind.LBracket -> {
                    bump()
                    val index = parsePipe()
                    if (bump()?.kind != CodeTokenKind.RBracket) {
                        val span = peekSpan()
                        throw ParseError(
                            message = "expected `]`",
                            file = file,
                            start = span.first,
                            length = span.second,
                        )
                    }
                    left = buildJsonObject {
                        put("type", "Expr.Index")
                        put("object", left)
                        put("index", index)
                    }
                }

                CodeTokenKind.LParen -> {
                    bump()
                    val args = mutableListOf<JsonElement>()
                    if (peekKind() != CodeTokenKind.RParen) {
                        while (true) {
                            args += parsePipe()
                            if (peekKind() == CodeTokenKind.Comma) {
                                bump()
                                continue
                            }
                            break
                        }
                    }
                    if (bump()?.kind != CodeTokenKind.RParen) {
                        val span = peekSpan()
                        throw ParseError(
                            message = "expected `)`",
                            file = file,
                            start = span.first,
                            length = span.second,
                        )
                    }
                    left = buildJsonObject {
                        put("type", "Expr.Call")
                        put("callee", left)
                        put("arguments", JsonArray(args))
                    }
                }

                else -> break
            }
        }
        return left
    }

    private fun parsePrimary(): JsonObject {
        val t = bump()
        if (t == null) {
            val span = peekSpan()
            throw ParseError(
                message = "unexpected end of expression",
                file = file,
                start = span.first,
                length = span.second,
            )
        }
        return when (t.kind) {
            CodeTokenKind.String -> buildJsonObject {
                put("type", "Expr.Literal")
                put("value", t.text)
            }

            CodeTokenKind.Bool -> buildJsonObject {
                put("type", "Expr.Literal")
                put("value", t.value == true)
            }

            CodeTokenKind.Null -> buildJsonObject {
                put("type", "Expr.Literal")
                put("value", JsonNull)
            }

            CodeTokenKind.Number -> {
                val num = t.text.toDoubleOrNull()
                    ?: throw ParseError(
                        message = "invalid number `${t.text}`",
                        file = file,
                        start = base + t.start,
                        length = t.end - t.start,
                    )
                buildJsonObject {
                    put("type", "Expr.Literal")
                    if (num % 1.0 == 0.0) put("value", num.toLong())
                    else put("value", num)
                }
            }

            CodeTokenKind.Ident -> buildJsonObject {
                put("type", "Expr.Identifier")
                put("name", t.text)
            }

            CodeTokenKind.LParen -> {
                val e = parsePipe()
                if (bump()?.kind != CodeTokenKind.RParen) {
                    val span = peekSpan()
                    throw ParseError(
                        message = "expected `)`",
                        file = file,
                        start = span.first,
                        length = span.second,
                    )
                }
                e
            }

            else -> throw ParseError(
                message = "unexpected token in expression",
                file = file,
                start = base + t.start,
                length = maxOf(1, t.end - t.start),
            )
        }
    }

    private fun bin(op: String, left: JsonObject, right: JsonObject) = buildJsonObject {
        put("type", "Expr.Binary")
        put("operator", op)
        put("left", left)
        put("right", right)
    }

    companion object {
        fun parse(
            input: String,
            source: String = input,
            file: String = "template.dejavu",
            base: Int = 0,
        ): JsonObject = ExprParser(source, file, base, input).parse()
    }
}

fun parseExpr(
    input: String,
    source: String = input,
    file: String = "template.dejavu",
    base: Int = 0,
): JsonObject = ExprParser.parse(input, source = source, file = file, base = base)
