package dejavu.language

/**
 * Hand-written lexer for code inside `<% ... %>`.
 * Character-class scanning belongs here — not in the parser.
 */
object CodeLexer {
    fun lexCode(
        input: String,
        source: String = input,
        file: String = "template.dejavu",
        base: Int = 0,
    ): List<CodeToken> {
        val tokens = mutableListOf<CodeToken>()
        var i = 0
        val n = input.length

        fun push(kind: CodeTokenKind, start: Int, end: Int, text: String = "", value: Boolean? = null) {
            tokens += CodeToken(kind = kind, text = text, value = value, start = start, end = end)
        }

        fun fail(start: Int, length: Int, message: String): Nothing {
            throw ParseError(
                message = message,
                file = file,
                start = base + start,
                length = length,
                label = "bad token",
            )
        }

        while (i < n) {
            val c = input[i].code
            // whitespace
            if (c == 0x20 || c == 0x09 || c == 0x0a || c == 0x0d) {
                i++
                continue
            }

            // two-char ops
            if (i + 1 < n) {
                val two = input.substring(i, i + 2)
                val kind2 = when (two) {
                    "%>" -> CodeTokenKind.CodeEnd
                    "|>" -> CodeTokenKind.PipeOp
                    "||" -> CodeTokenKind.OrOr
                    "&&" -> CodeTokenKind.AndAnd
                    "==" -> CodeTokenKind.EqEq
                    "!=" -> CodeTokenKind.NotEq
                    "<=" -> CodeTokenKind.LessEq
                    ">=" -> CodeTokenKind.GreaterEq
                    else -> null
                }
                if (kind2 != null) {
                    push(kind2, i, i + 2, two)
                    i += 2
                    continue
                }
            }

            val one = input[i]
            val kind1 = when (one) {
                '<' -> CodeTokenKind.Less
                '>' -> CodeTokenKind.Greater
                '+' -> CodeTokenKind.Plus
                '-' -> CodeTokenKind.Minus
                '*' -> CodeTokenKind.Star
                '/' -> CodeTokenKind.Slash
                '%' -> CodeTokenKind.Percent
                '!' -> CodeTokenKind.Bang
                '.' -> CodeTokenKind.Dot
                ',' -> CodeTokenKind.Comma
                '(' -> CodeTokenKind.LParen
                ')' -> CodeTokenKind.RParen
                '[' -> CodeTokenKind.LBracket
                ']' -> CodeTokenKind.RBracket
                else -> null
            }
            if (kind1 != null) {
                push(kind1, i, i + 1, one.toString())
                i++
                continue
            }

            // string
            if (one == '"' || one == '\'') {
                val quote = one
                val start = i
                i++
                val text = StringBuilder()
                var closed = false
                while (i < n) {
                    val ch = input[i]
                    if (ch == '\\') {
                        if (i + 1 >= n) fail(start, i - start + 1, "unterminated string escape")
                        text.append(input[i + 1])
                        i += 2
                        continue
                    }
                    if (ch == quote) {
                        i++
                        push(CodeTokenKind.String, start, i, text.toString())
                        closed = true
                        break
                    }
                    text.append(ch)
                    i++
                }
                if (!closed) {
                    fail(start, maxOf(1, i - start), "unterminated string")
                }
                continue
            }

            // number
            if (c in 0x30..0x39) {
                val start = i
                i++
                while (i < n) {
                    val d = input[i].code
                    if (d in 0x30..0x39) i++ else break
                }
                if (i < n && input[i] == '.') {
                    i++
                    while (i < n) {
                        val d = input[i].code
                        if (d in 0x30..0x39) i++ else break
                    }
                }
                push(CodeTokenKind.Number, start, i, input.substring(start, i))
                continue
            }

            // ident / keywords
            if (isIdentStart(c)) {
                val start = i
                i++
                while (i < n && isIdentContinue(input[i].code)) i++
                val text = input.substring(start, i)
                when (text) {
                    "true" -> push(CodeTokenKind.Bool, start, i, text, true)
                    "false" -> push(CodeTokenKind.Bool, start, i, text, false)
                    "null" -> push(CodeTokenKind.Null, start, i, text, null)
                    "in" -> push(CodeTokenKind.In, start, i, text)
                    else -> push(CodeTokenKind.Ident, start, i, text)
                }
                continue
            }

            fail(i, 1, "invalid token in expression (${one.toString().let { "\"$it\"" }})")
        }

        return tokens
    }

    private fun isIdentStart(c: Int): Boolean =
        (c in 0x41..0x5a) || (c in 0x61..0x7a) || c == 0x5f

    private fun isIdentContinue(c: Int): Boolean =
        isIdentStart(c) || (c in 0x30..0x39)
}
