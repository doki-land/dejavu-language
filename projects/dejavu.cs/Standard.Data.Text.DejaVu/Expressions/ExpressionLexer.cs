using System.Globalization;
using Std.Data.Text.Parsing;

namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     表达式词法分析器
/// </summary>
public sealed class ExpressionLexer
{
    private readonly string _source;
    private int _position;


    /// <summary>
    ///     创建表达式词法分析器
    /// </summary>
    /// <param name="source">表达式源码。</param>
    public ExpressionLexer(string source)
    {
        _source = source;
        _position = 0;
    }


    /// <summary>
    ///     将表达式源码分词
    /// </summary>
    public List<ExpressionToken> tokenize()
    {
        var tokens = new List<ExpressionToken>();

        while (_position < _source.Length)
        {
            skip_whitespace();

            if (_position >= _source.Length) break;

            var token = read_token();
            if (token != null) tokens.Add(token);
        }

        return tokens;
    }

    private void skip_whitespace()
    {
        while (_position < _source.Length && char.IsWhiteSpace(_source[_position])) _position++;
    }

    private ExpressionToken? read_token()
    {
        var ch = _source[_position];

        if (ch is '"' or '\'') return read_string();

        if (ch is >= '0' and <= '9' ||
            (ch == '.' && _position + 1 < _source.Length && char.IsDigit(_source[_position + 1]))) return read_number();

        if (ch == '+') return create_token(ExpressionTokenType.plus, "+");

        if (ch == '-') return create_token(ExpressionTokenType.minus, "-");

        if (ch == '*') return create_token(ExpressionTokenType.multiply, "*");

        if (ch == '/') return create_token(ExpressionTokenType.divide, "/");

        if (ch == '%') return create_token(ExpressionTokenType.modulo, "%");

        if (ch == '(') return create_token(ExpressionTokenType.left_paren, "(");

        if (ch == ')') return create_token(ExpressionTokenType.right_paren, ")");

        if (ch == '[') return create_token(ExpressionTokenType.left_bracket, "[");

        if (ch == ']') return create_token(ExpressionTokenType.right_bracket, "]");

        if (ch == ',') return create_token(ExpressionTokenType.comma, ",");

        if (ch == ':') return create_token(ExpressionTokenType.colon, ":");

        if (ch == '.') return create_token(ExpressionTokenType.dot, ".");

        if (ch == '=' && peek(1) == '=') return create_token(ExpressionTokenType.equal, "==");

        if (ch == '!' && peek(1) == '=') return create_token(ExpressionTokenType.not_equal, "!=");

        if (ch == '<' && peek(1) == '=') return create_token(ExpressionTokenType.less_than_or_equal, "<=");

        if (ch == '>' && peek(1) == '=') return create_token(ExpressionTokenType.greater_than_or_equal, ">=");

        if (ch == '<') return create_token(ExpressionTokenType.less_than, "<");

        if (ch == '>') return create_token(ExpressionTokenType.greater_than, ">");

        if (ch == '&' && peek(1) == '&') return create_token(ExpressionTokenType.and, "&&");

        if (ch == '|' && peek(1) == '|') return create_token(ExpressionTokenType.or, "||");

        if (ch == '|' && peek(1) == '>') return create_token(ExpressionTokenType.pipe, "|>");

        if (ch == '!') return create_token(ExpressionTokenType.not, "!");

        if (char.IsLetter(ch) || ch == '_') return read_identifier_or_keyword();

        throw new ParseException($"Unexpected character: {ch}");
    }

    private ExpressionToken create_token(ExpressionTokenType type, string text)
    {
        _position += text.Length;
        return new ExpressionToken(type, text);
    }

    private char peek(int offset)
    {
        var index = _position + offset;
        return index < _source.Length ? _source[index] : '\0';
    }

    private ExpressionToken read_string()
    {
        var quote = _source[_position];
        _position++;
        var start = _position;

        while (_position < _source.Length && _source[_position] != quote)
            if (_source[_position] == '\\' && _position + 1 < _source.Length)
                _position += 2;
            else
                _position++;

        var value = _source[start.._position];
        _position++; // 跳过结束引号

        return new ExpressionToken(ExpressionTokenType.@string, value);
    }

    private ExpressionToken read_number()
    {
        var start = _position;
        var hasDecimal = false;

        while (_position < _source.Length &&
               (char.IsDigit(_source[_position]) || _source[_position] == '.'))
        {
            if (_source[_position] == '.')
            {
                if (hasDecimal) break;

                hasDecimal = true;
            }

            _position++;
        }

        var text = _source[start.._position];
        var value = double.Parse(text, CultureInfo.InvariantCulture);

        return new ExpressionToken(ExpressionTokenType.number, value);
    }

    private ExpressionToken read_identifier_or_keyword()
    {
        var start = _position;

        while (_position < _source.Length &&
               (char.IsLetterOrDigit(_source[_position]) || _source[_position] == '_'))
            _position++;

        var text = _source[start.._position];

        return text.ToLowerInvariant() switch
        {
            "true" => new ExpressionToken(ExpressionTokenType.boolean, true),
            "false" => new ExpressionToken(ExpressionTokenType.boolean, false),
            _ => new ExpressionToken(ExpressionTokenType.identifier, text)
        };
    }
}