using System.Globalization;
using System.Text.Json.Nodes;

namespace Dejavu.Language;

/// <summary>Pratt-style expression parser over a <see cref="CodeToken"/> stream.</summary>
public sealed class ExprParser
{
    readonly string _file;
    readonly int _base;
    readonly List<CodeToken> _tokens;
    int _pos;

    public ExprParser(string input, string? source = null, string file = "template.dejavu", int @base = 0)
    {
        _file = file;
        _base = @base;
        _tokens = CodeLexer.Lex(input, source ?? input, file, @base);
        foreach (var t in _tokens)
        {
            if (t.Kind == CodeTokenKind.CodeEnd)
            {
                throw new ParseError(
                    "unexpected `%>` inside expression",
                    @base + t.Start,
                    t.End - t.Start,
                    file);
            }
        }
    }

    public static JsonNode Parse(string input, string? source = null, string file = "template.dejavu", int @base = 0)
        => new ExprParser(input, source, file, @base).Parse();

    public JsonNode Parse()
    {
        var expr = ParsePipe();
        if (_pos != _tokens.Count)
        {
            var span = PeekSpan();
            throw new ParseError("trailing input in expression", span.start, span.length, _file, "unexpected");
        }
        return expr;
    }

    CodeToken? Peek() => _pos < _tokens.Count ? _tokens[_pos] : null;

    CodeTokenKind? PeekKind() => Peek()?.Kind;

    (int start, int length) PeekSpan()
    {
        if (_pos < _tokens.Count)
        {
            var t = _tokens[_pos];
            return (_base + t.Start, Math.Max(1, t.End - t.Start));
        }
        var end = _base + (_tokens.Count > 0 ? _tokens[^1].End : 0);
        return (end, 1);
    }

    CodeToken? Bump() => _pos < _tokens.Count ? _tokens[_pos++] : null;

    string ExpectIdent()
    {
        var t = Bump();
        if (t is { Kind: CodeTokenKind.Ident } tok) return tok.Text;
        var span = PeekSpan();
        throw new ParseError("expected identifier", span.start, span.length, _file, "expected ident");
    }

    JsonNode ParsePipe()
    {
        var left = ParseOr();
        while (PeekKind() == CodeTokenKind.PipeOp)
        {
            Bump();
            var filter = ExpectIdent();
            var args = new JsonArray();
            if (PeekKind() == CodeTokenKind.LParen)
            {
                Bump();
                if (PeekKind() != CodeTokenKind.RParen)
                {
                    while (true)
                    {
                        args.Add(ParsePipe());
                        if (PeekKind() == CodeTokenKind.Comma)
                        {
                            Bump();
                            continue;
                        }
                        break;
                    }
                }
                if (Bump()?.Kind != CodeTokenKind.RParen)
                {
                    var span = PeekSpan();
                    throw new ParseError("expected `)` after filter arguments", span.start, span.length, _file);
                }
            }
            left = new JsonObject
            {
                ["type"] = "Expr.Pipe",
                ["expression"] = left,
                ["filter"] = filter,
                ["arguments"] = args,
            };
        }
        return left;
    }

    JsonNode ParseOr()
    {
        var left = ParseAnd();
        while (PeekKind() == CodeTokenKind.OrOr)
        {
            Bump();
            left = Bin("||", left, ParseAnd());
        }
        return left;
    }

    JsonNode ParseAnd()
    {
        var left = ParseCmp();
        while (PeekKind() == CodeTokenKind.AndAnd)
        {
            Bump();
            left = Bin("&&", left, ParseCmp());
        }
        return left;
    }

    JsonNode ParseCmp()
    {
        var left = ParseAdd();
        var op = PeekKind() switch
        {
            CodeTokenKind.EqEq => "==",
            CodeTokenKind.NotEq => "!=",
            CodeTokenKind.LessEq => "<=",
            CodeTokenKind.GreaterEq => ">=",
            CodeTokenKind.Less => "<",
            CodeTokenKind.Greater => ">",
            CodeTokenKind.In => "in",
            _ => null,
        };
        if (op is not null)
        {
            Bump();
            return Bin(op, left, ParseAdd());
        }
        return left;
    }

    JsonNode ParseAdd()
    {
        var left = ParseMul();
        while (true)
        {
            var kind = PeekKind();
            if (kind is CodeTokenKind.Plus or CodeTokenKind.Minus)
            {
                var op = kind == CodeTokenKind.Plus ? "+" : "-";
                Bump();
                left = Bin(op, left, ParseMul());
            }
            else break;
        }
        return left;
    }

    JsonNode ParseMul()
    {
        var left = ParseUnary();
        while (true)
        {
            var kind = PeekKind();
            if (kind is CodeTokenKind.Star or CodeTokenKind.Slash or CodeTokenKind.Percent)
            {
                var op = kind == CodeTokenKind.Star ? "*" : kind == CodeTokenKind.Slash ? "/" : "%";
                Bump();
                left = Bin(op, left, ParseUnary());
            }
            else break;
        }
        return left;
    }

    JsonNode ParseUnary()
    {
        var kind = PeekKind();
        if (kind is CodeTokenKind.Bang or CodeTokenKind.Minus or CodeTokenKind.Plus)
        {
            var op = kind == CodeTokenKind.Bang ? "!" : kind == CodeTokenKind.Minus ? "-" : "+";
            Bump();
            return new JsonObject
            {
                ["type"] = "Expr.Unary",
                ["operator"] = op,
                ["argument"] = ParseUnary(),
            };
        }
        return ParsePostfix();
    }

    JsonNode ParsePostfix()
    {
        var left = ParsePrimary();
        while (true)
        {
            var kind = PeekKind();
            if (kind == CodeTokenKind.Dot)
            {
                Bump();
                left = new JsonObject
                {
                    ["type"] = "Expr.Member",
                    ["object"] = left,
                    ["property"] = ExpectIdent(),
                };
            }
            else if (kind == CodeTokenKind.LBracket)
            {
                Bump();
                var index = ParsePipe();
                if (Bump()?.Kind != CodeTokenKind.RBracket)
                {
                    var span = PeekSpan();
                    throw new ParseError("expected `]`", span.start, span.length, _file);
                }
                left = new JsonObject
                {
                    ["type"] = "Expr.Index",
                    ["object"] = left,
                    ["index"] = index,
                };
            }
            else if (kind == CodeTokenKind.LParen)
            {
                Bump();
                var args = new JsonArray();
                if (PeekKind() != CodeTokenKind.RParen)
                {
                    while (true)
                    {
                        args.Add(ParsePipe());
                        if (PeekKind() == CodeTokenKind.Comma)
                        {
                            Bump();
                            continue;
                        }
                        break;
                    }
                }
                if (Bump()?.Kind != CodeTokenKind.RParen)
                {
                    var span = PeekSpan();
                    throw new ParseError("expected `)`", span.start, span.length, _file);
                }
                left = new JsonObject
                {
                    ["type"] = "Expr.Call",
                    ["callee"] = left,
                    ["arguments"] = args,
                };
            }
            else break;
        }
        return left;
    }

    JsonNode ParsePrimary()
    {
        var t = Bump();
        if (t is null)
        {
            var span = PeekSpan();
            throw new ParseError("unexpected end of expression", span.start, span.length, _file);
        }

        switch (t.Value.Kind)
        {
            case CodeTokenKind.String:
                return new JsonObject { ["type"] = "Expr.Literal", ["value"] = t.Value.Text };
            case CodeTokenKind.Bool:
                return new JsonObject { ["type"] = "Expr.Literal", ["value"] = t.Value.Value == true };
            case CodeTokenKind.Null:
                return new JsonObject { ["type"] = "Expr.Literal", ["value"] = null };
            case CodeTokenKind.Number:
            {
                if (!double.TryParse(t.Value.Text, NumberStyles.Float, CultureInfo.InvariantCulture, out var num))
                {
                    throw new ParseError(
                        $"invalid number `{t.Value.Text}`",
                        _base + t.Value.Start,
                        t.Value.End - t.Value.Start,
                        _file);
                }
                return new JsonObject { ["type"] = "Expr.Literal", ["value"] = num };
            }
            case CodeTokenKind.Ident:
                return new JsonObject { ["type"] = "Expr.Identifier", ["name"] = t.Value.Text };
            case CodeTokenKind.LParen:
            {
                var e = ParsePipe();
                if (Bump()?.Kind != CodeTokenKind.RParen)
                {
                    var span = PeekSpan();
                    throw new ParseError("expected `)`", span.start, span.length, _file);
                }
                return e;
            }
            default:
                throw new ParseError(
                    "unexpected token in expression",
                    _base + t.Value.Start,
                    Math.Max(1, t.Value.End - t.Value.Start),
                    _file);
        }
    }

    static JsonObject Bin(string op, JsonNode left, JsonNode right) => new()
    {
        ["type"] = "Expr.Binary",
        ["operator"] = op,
        ["left"] = left,
        ["right"] = right,
    };
}
