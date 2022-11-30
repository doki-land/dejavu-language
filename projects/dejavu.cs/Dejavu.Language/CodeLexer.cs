namespace Dejavu.Language;

/// <summary>
/// Hand-written lexer for code inside <c>&lt;% ... %&gt;</c>.
/// Character-class scanning belongs here — not in the parser.
/// </summary>
public static class CodeLexer
{
    public static List<CodeToken> Lex(
        string input,
        string? source = null,
        string file = "template.dejavu",
        int @base = 0)
    {
        _ = source;
        var tokens = new List<CodeToken>();
        var i = 0;
        var n = input.Length;

        void Push(CodeTokenKind kind, int start, int end, string text = "", bool? value = null)
            => tokens.Add(new CodeToken(kind, text, value, start, end));

        void Fail(int start, int length, string message)
            => throw new ParseError(message, @base + start, length, file, "bad token");

        while (i < n)
        {
            var c = input[i];
            if (c is ' ' or '\t' or '\n' or '\r')
            {
                i++;
                continue;
            }

            if (i + 1 < n)
            {
                var two = input.AsSpan(i, 2);
                CodeTokenKind? kind2 = two switch
                {
                    "%>" => CodeTokenKind.CodeEnd,
                    "|>" => CodeTokenKind.PipeOp,
                    "||" => CodeTokenKind.OrOr,
                    "&&" => CodeTokenKind.AndAnd,
                    "==" => CodeTokenKind.EqEq,
                    "!=" => CodeTokenKind.NotEq,
                    "<=" => CodeTokenKind.LessEq,
                    ">=" => CodeTokenKind.GreaterEq,
                    _ => null,
                };
                if (kind2 is { } k2)
                {
                    Push(k2, i, i + 2, input.Substring(i, 2));
                    i += 2;
                    continue;
                }
            }

            CodeTokenKind? kind1 = c switch
            {
                '<' => CodeTokenKind.Less,
                '>' => CodeTokenKind.Greater,
                '+' => CodeTokenKind.Plus,
                '-' => CodeTokenKind.Minus,
                '*' => CodeTokenKind.Star,
                '/' => CodeTokenKind.Slash,
                '%' => CodeTokenKind.Percent,
                '!' => CodeTokenKind.Bang,
                '.' => CodeTokenKind.Dot,
                ',' => CodeTokenKind.Comma,
                '(' => CodeTokenKind.LParen,
                ')' => CodeTokenKind.RParen,
                '[' => CodeTokenKind.LBracket,
                ']' => CodeTokenKind.RBracket,
                _ => null,
            };
            if (kind1 is { } k1)
            {
                Push(k1, i, i + 1, c.ToString());
                i++;
                continue;
            }

            if (c is '"' or '\'')
            {
                var quote = c;
                var start = i;
                i++;
                var text = new System.Text.StringBuilder();
                var closed = false;
                while (i < n)
                {
                    var ch = input[i];
                    if (ch == '\\')
                    {
                        if (i + 1 >= n) Fail(start, i - start + 1, "unterminated string escape");
                        text.Append(input[i + 1]);
                        i += 2;
                        continue;
                    }
                    if (ch == quote)
                    {
                        i++;
                        Push(CodeTokenKind.String, start, i, text.ToString());
                        closed = true;
                        break;
                    }
                    text.Append(ch);
                    i++;
                }
                if (!closed)
                    Fail(start, Math.Max(1, i - start), "unterminated string");
                continue;
            }

            if (c is >= '0' and <= '9')
            {
                var start = i;
                i++;
                while (i < n && input[i] is >= '0' and <= '9') i++;
                if (i < n && input[i] == '.')
                {
                    i++;
                    while (i < n && input[i] is >= '0' and <= '9') i++;
                }
                Push(CodeTokenKind.Number, start, i, input[start..i]);
                continue;
            }

            if (IsIdentStart(c))
            {
                var start = i;
                i++;
                while (i < n && IsIdentContinue(input[i])) i++;
                var text = input[start..i];
                if (text == "true") Push(CodeTokenKind.Bool, start, i, text, true);
                else if (text == "false") Push(CodeTokenKind.Bool, start, i, text, false);
                else if (text == "null") Push(CodeTokenKind.Null, start, i, text, null);
                else if (text == "in") Push(CodeTokenKind.In, start, i, text);
                else Push(CodeTokenKind.Ident, start, i, text);
                continue;
            }

            Fail(i, 1, $"invalid token in expression ({System.Text.Json.JsonSerializer.Serialize(c.ToString())})");
        }

        return tokens;
    }

    static bool IsIdentStart(char c) =>
        c is (>= 'A' and <= 'Z') or (>= 'a' and <= 'z') or '_';

    static bool IsIdentContinue(char c) =>
        IsIdentStart(c) || c is >= '0' and <= '9';
}
