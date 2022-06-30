namespace Dejavu.Language;

/// <summary>Classify a code block from its token stream (no string-prefix hacks).</summary>
public abstract record CodeHead
{
    public sealed record If(string TestSlice, int TestAbs) : CodeHead;
    public sealed record Loop(string Item, string IterSlice, int IterAbs) : CodeHead;
    public sealed record ElseIf(string TestSlice, int TestAbs) : CodeHead;
    public sealed record Else : CodeHead;
    public sealed record EndIf : CodeHead;
    public sealed record EndLoop : CodeHead;
    public sealed record Expr(string ExprSlice, int ExprAbs) : CodeHead;

    public string KindName => this switch
    {
        If => "if",
        Loop => "loop",
        ElseIf => "else_if",
        Else => "else",
        EndIf => "end_if",
        EndLoop => "end_loop",
        Expr => "expr",
        _ => "unknown",
    };

    public static (string Slice, int Abs) SliceFromTokens(
        string content,
        int contentBase,
        IReadOnlyList<CodeToken> tokens)
    {
        if (tokens.Count == 0) return ("", contentBase);
        var start = tokens[0].Start;
        var end = tokens[^1].End;
        return (content[start..end], contentBase + start);
    }

    public static CodeHead Classify(string content, int contentBase, IReadOnlyList<CodeToken> tokens)
    {
        if (tokens.Count == 0)
            return new Expr("", contentBase);

        var t0 = tokens[0];
        if (t0.Kind == CodeTokenKind.Ident && t0.Text == "if")
        {
            var (slice, abs) = SliceFromTokens(content, contentBase, Slice(tokens, 1));
            return new If(slice, abs);
        }

        if (t0.Kind == CodeTokenKind.Ident && t0.Text == "loop")
        {
            if (tokens.Count < 2 || tokens[1].Kind != CodeTokenKind.Ident)
            {
                throw new ParseError(
                    "loop requires item identifier",
                    contentBase + t0.Start,
                    Math.Max(1, (tokens.Count > 0 ? tokens[^1].End : t0.End) - t0.Start));
            }
            if (tokens.Count < 3 || tokens[2].Kind != CodeTokenKind.In)
            {
                throw new ParseError(
                    "loop requires `in`",
                    contentBase + t0.Start,
                    Math.Max(1, tokens[^1].End - t0.Start),
                    label: "expected `item in iterable`");
            }
            var item = tokens[1].Text;
            var (slice, abs) = SliceFromTokens(content, contentBase, Slice(tokens, 3));
            return new Loop(item, slice, abs);
        }

        if (t0.Kind == CodeTokenKind.Ident && t0.Text == "else")
        {
            if (tokens.Count >= 2 && tokens[1].Kind == CodeTokenKind.Ident && tokens[1].Text == "if")
            {
                var (slice, abs) = SliceFromTokens(content, contentBase, Slice(tokens, 2));
                return new ElseIf(slice, abs);
            }
            if (tokens.Count == 1) return new Else();
            throw new ParseError(
                "unexpected tokens after `else`",
                contentBase + tokens[1].Start,
                1);
        }

        if (t0.Kind == CodeTokenKind.Ident && t0.Text == "end")
        {
            if (tokens.Count == 2 && tokens[1].Kind == CodeTokenKind.Ident && tokens[1].Text == "if")
                return new EndIf();
            if (tokens.Count == 2 && tokens[1].Kind == CodeTokenKind.Ident && tokens[1].Text == "loop")
                return new EndLoop();
            throw new ParseError(
                "expected `end if` or `end loop`",
                contentBase + t0.Start,
                Math.Max(1, tokens[^1].End - t0.Start));
        }

        var (exprSlice, exprAbs) = SliceFromTokens(content, contentBase, tokens);
        return new Expr(exprSlice, exprAbs);
    }

    static IReadOnlyList<CodeToken> Slice(IReadOnlyList<CodeToken> tokens, int start)
    {
        if (start >= tokens.Count) return Array.Empty<CodeToken>();
        var list = new List<CodeToken>(tokens.Count - start);
        for (var i = start; i < tokens.Count; i++) list.Add(tokens[i]);
        return list;
    }
}
