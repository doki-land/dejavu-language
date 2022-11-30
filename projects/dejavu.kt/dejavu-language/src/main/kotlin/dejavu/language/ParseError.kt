package dejavu.language

/** Span-aware parse failure. */
class ParseError(
    message: String,
    val file: String = "template.dejavu",
    val start: Int,
    length: Int = 1,
    val label: String = "here",
) : RuntimeException(message) {
    val length: Int = maxOf(1, length)
}
