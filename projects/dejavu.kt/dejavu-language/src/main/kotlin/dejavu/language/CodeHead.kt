package dejavu.language

/** Classify a code block from its token stream (no string-prefix hacks). */
sealed class CodeHead {
    data class If(val testSlice: String, val testAbs: Int) : CodeHead()
    data class Loop(val item: String, val iterSlice: String, val iterAbs: Int) : CodeHead()
    data class ElseIf(val testSlice: String, val testAbs: Int) : CodeHead()
    data object Else : CodeHead()
    data object EndIf : CodeHead()
    data object EndLoop : CodeHead()
    data class Expr(val exprSlice: String, val exprAbs: Int) : CodeHead()

    val kind: Kind
        get() = when (this) {
            is If -> Kind.If
            is Loop -> Kind.Loop
            is ElseIf -> Kind.ElseIf
            Else -> Kind.Else
            EndIf -> Kind.EndIf
            EndLoop -> Kind.EndLoop
            is Expr -> Kind.Expr
        }

    enum class Kind {
        If,
        Loop,
        ElseIf,
        Else,
        EndIf,
        EndLoop,
        Expr,
    }
}

private fun sliceFromTokens(
    content: String,
    contentBase: Int,
    tokens: List<CodeToken>,
): Pair<String, Int> {
    if (tokens.isEmpty()) return "" to contentBase
    val start = tokens.first().start
    val end = tokens.last().end
    return content.substring(start, end) to (contentBase + start)
}

fun classifyCode(content: String, contentBase: Int, tokens: List<CodeToken>): CodeHead {
    if (tokens.isEmpty()) {
        return CodeHead.Expr(exprSlice = "", exprAbs = contentBase)
    }
    val t0 = tokens[0]
    if (t0.kind == CodeTokenKind.Ident && t0.text == "if") {
        val (slice, abs) = sliceFromTokens(content, contentBase, tokens.drop(1))
        return CodeHead.If(testSlice = slice, testAbs = abs)
    }
    if (t0.kind == CodeTokenKind.Ident && t0.text == "loop") {
        if (tokens.getOrNull(1)?.kind != CodeTokenKind.Ident) {
            throw ParseError(
                message = "loop requires item identifier",
                start = contentBase + t0.start,
                length = maxOf(1, (tokens.lastOrNull()?.end ?: t0.end) - t0.start),
            )
        }
        if (tokens.getOrNull(2)?.kind != CodeTokenKind.In) {
            throw ParseError(
                message = "loop requires `in`",
                start = contentBase + t0.start,
                length = maxOf(1, (tokens.lastOrNull()?.end ?: t0.end) - t0.start),
                label = "expected `item in iterable`",
            )
        }
        val item = tokens[1].text
        val (slice, abs) = sliceFromTokens(content, contentBase, tokens.drop(3))
        return CodeHead.Loop(item = item, iterSlice = slice, iterAbs = abs)
    }
    if (t0.kind == CodeTokenKind.Ident && t0.text == "else") {
        val t1 = tokens.getOrNull(1)
        if (t1?.kind == CodeTokenKind.Ident && t1.text == "if") {
            val (slice, abs) = sliceFromTokens(content, contentBase, tokens.drop(2))
            return CodeHead.ElseIf(testSlice = slice, testAbs = abs)
        }
        if (tokens.size == 1) return CodeHead.Else
        throw ParseError(
            message = "unexpected tokens after `else`",
            start = contentBase + tokens[1].start,
            length = 1,
        )
    }
    if (t0.kind == CodeTokenKind.Ident && t0.text == "end") {
        val t1 = tokens.getOrNull(1)
        if (t1?.kind == CodeTokenKind.Ident && t1.text == "if" && tokens.size == 2) {
            return CodeHead.EndIf
        }
        if (t1?.kind == CodeTokenKind.Ident && t1.text == "loop" && tokens.size == 2) {
            return CodeHead.EndLoop
        }
        throw ParseError(
            message = "expected `end if` or `end loop`",
            start = contentBase + t0.start,
            length = maxOf(1, (tokens.lastOrNull()?.end ?: t0.end) - t0.start),
        )
    }
    val (slice, abs) = sliceFromTokens(content, contentBase, tokens)
    return CodeHead.Expr(exprSlice = slice, exprAbs = abs)
}
