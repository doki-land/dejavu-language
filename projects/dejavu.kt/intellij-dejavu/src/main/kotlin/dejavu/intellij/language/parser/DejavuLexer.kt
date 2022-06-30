package dejavu.intellij.language.parser

import com.intellij.lexer.LexerBase
import com.intellij.lexer.LexerPosition
import com.intellij.psi.TokenType
import com.intellij.psi.tree.IElementType
import dejavu.intellij.language.TemplateConfig
import dejavu.intellij.language.psi.DejavuTypes

/**
 * Dejavu 模板语言词法分析器
 * 支持多种空白控制符：
 * - <%= : 输出表达式
 * - <%_ : 去除前面空白
 * - <%- : 去除后面空白
 * - <%~ : 去除前后空白
 * - _%> : 去除前面空白（结束）
 * - -%> : 去除后面空白（结束）
 * - ~%> : 去除前后空白（结束）
 */
class DejavuLexer(private val config: TemplateConfig) : LexerBase() {
    private lateinit var buffer: CharSequence
    private var startOffset: Int = 0
    private var endOffset: Int = 0
    private var currentOffset: Int = 0
    private var tokenStart: Int = 0
    private var tokenEnd: Int = 0
    private var currentTokenType: IElementType? = null
    private var state: Int = 0

    companion object {
        private const val STATE_NORMAL = 0
        private const val STATE_EXPR = 1
        private const val STATE_COMMENT = 2
    }

    override fun start(buffer: CharSequence, startOffset: Int, endOffset: Int, initialState: Int) {
        this.buffer = buffer
        this.startOffset = startOffset
        this.endOffset = endOffset
        this.currentOffset = startOffset
        this.tokenStart = startOffset
        this.tokenEnd = startOffset
        this.currentTokenType = null
        this.state = initialState
        advance()
    }

    override fun getState(): Int {
        return state
    }

    override fun getTokenType(): IElementType? {
        return currentTokenType
    }

    override fun getTokenStart(): Int {
        return tokenStart
    }

    override fun getTokenEnd(): Int {
        return tokenEnd
    }

    override fun advance() {
        tokenStart = currentOffset
        currentTokenType = null

        if (currentOffset >= endOffset) {
            currentTokenType = null
            tokenEnd = endOffset
            state = STATE_NORMAL
            return
        }

        when (state) {
            STATE_NORMAL -> advanceNormal()
            STATE_EXPR -> advanceExpr()
            STATE_COMMENT -> advanceComment()
        }
    }

    private fun advanceNormal() {
        // 尝试匹配各种开始定界符（按长度优先）
        val slotStartResult = tryParseSlotStart()
        if (slotStartResult != null) {
            currentTokenType = DejavuTypes.SLOT_L
            currentOffset += slotStartResult.second.length
            tokenEnd = currentOffset
            state = STATE_EXPR
            return
        }

        if (tryParseDelimiter(config.commentStart, DejavuTypes.COMMENT_L)) {
            state = STATE_COMMENT
            return
        }

        parseText()
    }

    private fun advanceExpr() {
        // 首先尝试匹配各种结束定界符（这必须在其他检查之前）
        val slotEndResult = tryParseSlotEnd()
        if (slotEndResult != null) {
            currentTokenType = DejavuTypes.SLOT_R
            currentOffset += slotEndResult.second.length
            tokenEnd = currentOffset
            state = STATE_NORMAL
            return
        }

        // 跳过空白
        if (buffer[currentOffset].isWhitespace()) {
            currentTokenType = DejavuTypes.WHITESPACE
            while (currentOffset < endOffset && buffer[currentOffset].isWhitespace()) {
                currentOffset++
            }
            tokenEnd = currentOffset
            return
        }

        // 检查是否是关键词
        val keywordResult = tryParseKeyword()
        if (keywordResult != null) {
            currentTokenType = keywordResult.first
            currentOffset += keywordResult.second.length
            tokenEnd = currentOffset
            return
        }

        // 检查是否是字符串
        if (buffer[currentOffset] == '"' || buffer[currentOffset] == '\'') {
            currentTokenType = DejavuTypes.STRING
            val quote = buffer[currentOffset]
            currentOffset++
            while (currentOffset < endOffset && buffer[currentOffset] != quote) {
                currentOffset++
            }
            if (currentOffset < endOffset) {
                currentOffset++ // 消费结束引号
            }
            tokenEnd = currentOffset
            return
        }

        // 检查是否是数字
        if (buffer[currentOffset].isDigit()) {
            currentTokenType = DejavuTypes.NUMBER
            while (currentOffset < endOffset && (buffer[currentOffset].isDigit() || buffer[currentOffset] == '.')) {
                currentOffset++
            }
            tokenEnd = currentOffset
            return
        }

        // 检查是否是标识符
        if (buffer[currentOffset].isLetter() || buffer[currentOffset] == '_') {
            currentTokenType = DejavuTypes.IDENTIFIER
            while (currentOffset < endOffset && (buffer[currentOffset].isLetterOrDigit() || buffer[currentOffset] == '_')) {
                currentOffset++
            }
            tokenEnd = currentOffset
            return
        }

        // 检查是否是括号
        when (buffer[currentOffset]) {
            '(' -> {
                currentTokenType = DejavuTypes.LPAREN
                currentOffset++
                tokenEnd = currentOffset
                return
            }

            ')' -> {
                currentTokenType = DejavuTypes.RPAREN
                currentOffset++
                tokenEnd = currentOffset
                return
            }

            '[' -> {
                currentTokenType = DejavuTypes.LBRACKET
                currentOffset++
                tokenEnd = currentOffset
                return
            }

            ']' -> {
                currentTokenType = DejavuTypes.RBRACKET
                currentOffset++
                tokenEnd = currentOffset
                return
            }

            '.' -> {
                currentTokenType = DejavuTypes.DOT
                currentOffset++
                tokenEnd = currentOffset
                return
            }

            ',' -> {
                currentTokenType = DejavuTypes.COMMA
                currentOffset++
                tokenEnd = currentOffset
                return
            }
        }

        // 检查是否是操作符（但不包括 =，因为 =%> 是结束定界符）
        val currentChar = buffer[currentOffset]
        when (currentChar) {
            '+' -> {
                currentTokenType = DejavuTypes.PLUS
                currentOffset++
                // 检查 += 操作符
                if (currentOffset < endOffset && buffer[currentOffset] == '=') {
                    currentTokenType = DejavuTypes.PLUS_ASSIGN
                    currentOffset++
                }
                tokenEnd = currentOffset
                return
            }

            '-' -> {
                currentTokenType = DejavuTypes.MINUS
                currentOffset++
                // 检查 -= 操作符
                if (currentOffset < endOffset && buffer[currentOffset] == '=') {
                    currentTokenType = DejavuTypes.MINUS_ASSIGN
                    currentOffset++
                }
                tokenEnd = currentOffset
                return
            }

            '*' -> {
                currentTokenType = DejavuTypes.MULTIPLY
                currentOffset++
                // 检查 *= 操作符
                if (currentOffset < endOffset && buffer[currentOffset] == '=') {
                    currentTokenType = DejavuTypes.MULTIPLY_ASSIGN
                    currentOffset++
                }
                tokenEnd = currentOffset
                return
            }

            '/' -> {
                currentTokenType = DejavuTypes.DIVIDE
                currentOffset++
                // 检查 /= 操作符
                if (currentOffset < endOffset && buffer[currentOffset] == '=') {
                    currentTokenType = DejavuTypes.DIVIDE_ASSIGN
                    currentOffset++
                }
                tokenEnd = currentOffset
                return
            }

            '%' -> {
                currentTokenType = DejavuTypes.MODULO
                currentOffset++
                // 检查 %= 操作符
                if (currentOffset < endOffset && buffer[currentOffset] == '=') {
                    currentTokenType = DejavuTypes.MODULO_ASSIGN
                    currentOffset++
                }
                tokenEnd = currentOffset
                return
            }

            '<' -> {
                currentTokenType = DejavuTypes.LESS_THAN
                currentOffset++
                // 检查 <= 操作符
                if (currentOffset < endOffset && buffer[currentOffset] == '=') {
                    currentTokenType = DejavuTypes.LESS_THAN_OR_EQUAL
                    currentOffset++
                }
                tokenEnd = currentOffset
                return
            }

            '>' -> {
                currentTokenType = DejavuTypes.GREATER_THAN
                currentOffset++
                // 检查 >= 操作符
                if (currentOffset < endOffset && buffer[currentOffset] == '=') {
                    currentTokenType = DejavuTypes.GREATER_THAN_OR_EQUAL
                    currentOffset++
                }
                tokenEnd = currentOffset
                return
            }

            '!' -> {
                currentTokenType = DejavuTypes.NOT
                currentOffset++
                // 检查 != 操作符
                if (currentOffset < endOffset && buffer[currentOffset] == '=') {
                    currentTokenType = DejavuTypes.NOT_EQUAL
                    currentOffset++
                }
                tokenEnd = currentOffset
                return
            }

            '&' -> {
                currentOffset++
                // 检查 && 操作符
                if (currentOffset < endOffset && buffer[currentOffset] == '&') {
                    currentTokenType = DejavuTypes.AND
                    currentOffset++
                } else {
                    currentTokenType = DejavuTypes.PUNCTUATION
                }
                tokenEnd = currentOffset
                return
            }

            '|' -> {
                currentOffset++
                // 检查 || 操作符
                if (currentOffset < endOffset && buffer[currentOffset] == '|') {
                    currentTokenType = DejavuTypes.OR
                    currentOffset++
                } else {
                    currentTokenType = DejavuTypes.PUNCTUATION
                }
                tokenEnd = currentOffset
                return
            }

            '{' -> {
                currentTokenType = DejavuTypes.PUNCTUATION
                currentOffset++
                tokenEnd = currentOffset
                return
            }

            '}' -> {
                currentTokenType = DejavuTypes.PUNCTUATION
                currentOffset++
                tokenEnd = currentOffset
                return
            }
        }

        // 单独处理 = 字符（不作为操作符，避免与 =%> 冲突）
        if (currentChar == '=') {
            // 检查是否是 == 操作符
            if (currentOffset + 1 < endOffset && buffer[currentOffset + 1] == '=') {
                currentTokenType = DejavuTypes.EQUAL
                currentOffset += 2
                tokenEnd = currentOffset
                return
            }
            // 单独的 = 作为赋值操作符
            currentTokenType = DejavuTypes.ASSIGN
            currentOffset++
            tokenEnd = currentOffset
            return
        }

        // 其他字符作为标点符号处理
        currentTokenType = DejavuTypes.PUNCTUATION
        currentOffset++
        tokenEnd = currentOffset
    }

    private fun advanceComment() {
        if (tryParseDelimiter(config.commentEnd, DejavuTypes.COMMENT_R)) {
            state = STATE_NORMAL
            return
        }

        parseCommentContent()
    }

    /**
     * 尝试匹配各种开始定界符
     * @return 匹配到的 Token 类型和定界符字符串，如果没有匹配则返回 null
     */
    private fun tryParseSlotStart(): Pair<IElementType, String>? {
        // 基于 config.slotStart 构建所有可能的开始定界符
        val slotStart = config.slotStart
        if (slotStart.isEmpty()) return null

        val possibleStarts = listOf(
            slotStart + "~",  // 去除前后空白
            slotStart + "=",  // 输出表达式
            slotStart + "_",  // 去除前面空白
            slotStart + "-",  // 去除后面空白
            slotStart         // 基本
        )

        for (delimiter in possibleStarts) {
            if (delimiter.isEmpty()) continue
            if (currentOffset + delimiter.length <= endOffset &&
                buffer.substring(currentOffset, currentOffset + delimiter.length) == delimiter
            ) {
                return Pair(DejavuTypes.SLOT_L, delimiter)
            }
        }
        return null
    }

    /**
     * 尝试匹配各种结束定界符
     * @return 匹配到的 Token 类型和定界符字符串，如果没有匹配则返回 null
     */
    private fun tryParseSlotEnd(): Pair<IElementType, String>? {
        // 基于 config.slotEnd 构建所有可能的结束定界符
        val slotEnd = config.slotEnd
        if (slotEnd.isEmpty()) return null

        val possibleEnds = listOf(
            "=" + slotEnd,   // 输出表达式结束
            "~" + slotEnd,  // 去除前后空白
            "_" + slotEnd,  // 去除前面空白
            "-" + slotEnd,  // 去除后面空白
            slotEnd         // 基本
        )

        for (delimiter in possibleEnds) {
            if (delimiter.isEmpty()) continue
            if (currentOffset + delimiter.length <= endOffset &&
                buffer.substring(currentOffset, currentOffset + delimiter.length) == delimiter
            ) {
                return Pair(DejavuTypes.SLOT_R, delimiter)
            }
        }
        return null
    }

    private fun tryParseDelimiter(delimiter: String, tokenType: IElementType): Boolean {
        if (delimiter.isEmpty()) return false
        if (currentOffset + delimiter.length <= endOffset &&
            buffer.substring(currentOffset, currentOffset + delimiter.length) == delimiter
        ) {
            currentTokenType = tokenType
            currentOffset += delimiter.length
            tokenEnd = currentOffset
            return true
        }
        return false
    }

    /**
     * 尝试匹配关键词
     * @return 匹配到的 Token 类型和关键词字符串，如果没有匹配则返回 null
     */
    private fun tryParseKeyword(): Pair<IElementType, String>? {
        val keywords = mutableMapOf(
            "if" to DejavuTypes.KEYWORD_IF,
            "end" to DejavuTypes.KEYWORD_END,
            "loop" to DejavuTypes.KEYWORD_LOOP,
            "match" to DejavuTypes.KEYWORD_MATCH,
            "case" to DejavuTypes.KEYWORD_CASE,
            "else" to DejavuTypes.KEYWORD_ELSE,
            "while" to DejavuTypes.KEYWORD_WHILE,
            "until" to DejavuTypes.KEYWORD_UNTIL,
            "extends" to DejavuTypes.KEYWORD_EXTENDS,
            "block" to DejavuTypes.KEYWORD_BLOCK,
            "include" to DejavuTypes.KEYWORD_INCLUDE,
            "raw" to DejavuTypes.KEYWORD_RAW,
            "let" to DejavuTypes.KEYWORD_LET,
            "in" to DejavuTypes.KEYWORD_IN,
            "macro" to DejavuTypes.KEYWORD_MACRO
        )

        // 如果允许 legacy for，将 for 视为 loop
        if (config.allowLegacyFor) {
            keywords["for"] = DejavuTypes.KEYWORD_LOOP
        }

        for ((keyword, tokenType) in keywords) {
            if (currentOffset + keyword.length <= endOffset) {
                val candidate = buffer.substring(currentOffset, currentOffset + keyword.length)
                if (candidate == keyword) {
                    // 确保关键词后面是空格或结束定界符
                    val nextChar = if (currentOffset + keyword.length < endOffset) {
                        buffer[currentOffset + keyword.length]
                    } else {
                        ' '
                    }
                    if (!nextChar.isLetterOrDigit()) {
                        return Pair(tokenType, keyword)
                    }
                }
            }
        }
        return null
    }

    private fun parseCommentContent() {
        currentTokenType = DejavuTypes.COMMENT_CONTENT
        while (currentOffset < endOffset) {
            if (config.commentEnd.isNotEmpty() &&
                currentOffset + config.commentEnd.length <= endOffset &&
                buffer.substring(currentOffset, currentOffset + config.commentEnd.length) == config.commentEnd
            ) {
                break
            }
            currentOffset++
        }
        tokenEnd = currentOffset
    }

    private fun parseText() {
        currentTokenType = DejavuTypes.TEXT
        val start = currentOffset
        // 基于 config.slotStart 构建所有可能的开始定界符
        val slotStart = config.slotStart
        val possibleStarts = if (slotStart.isNotEmpty()) {
            listOf(
                slotStart + "~",  // 去除前后空白
                slotStart + "=",  // 输出表达式
                slotStart + "_",  // 去除前面空白
                slotStart + "-",  // 去除后面空白
                slotStart         // 基本
            )
        } else {
            emptyList()
        }
        while (currentOffset < endOffset) {
            // 检查是否匹配任何开始定界符
            for (delimiter in possibleStarts) {
                if (delimiter.isEmpty()) continue
                if (currentOffset + delimiter.length <= endOffset &&
                    buffer.substring(currentOffset, currentOffset + delimiter.length) == delimiter
                ) {
                    tokenEnd = currentOffset
                    return
                }
            }
            // 检查是否匹配注释开始定界符
            if (config.commentStart.isNotEmpty() &&
                currentOffset + config.commentStart.length <= endOffset &&
                buffer.substring(currentOffset, currentOffset + config.commentStart.length) == config.commentStart
            ) {
                tokenEnd = currentOffset
                return
            }
            currentOffset++
        }
        tokenEnd = currentOffset
    }

    override fun getBufferSequence(): CharSequence {
        return buffer
    }

    override fun getBufferEnd(): Int {
        return endOffset
    }
}
