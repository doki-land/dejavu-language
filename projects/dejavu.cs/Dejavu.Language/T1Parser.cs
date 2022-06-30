using System.Text.Json.Nodes;
using Dejavu.Types;

namespace Dejavu.Language;

/// <summary>T1 source → IR JSON document.</summary>
public static class T1Parser
{
    public static JsonObject ParseToIr(string source)
    {
        return new JsonObject
        {
            ["irVersion"] = "1.0",
            ["language"] = IrDefaults.DefaultLanguage.DeepClone(),
            ["body"] = new JsonObject
            {
                ["type"] = "Template",
                ["children"] = new TemplateParser(source).Parse(),
            },
        };
    }

    sealed class TemplateParser(string source, string file = "template.dejavu")
    {
        public JsonArray Parse()
        {
            var (children, _) = ParseBody(0, []);
            return children;
        }

        (JsonArray children, int i) ParseBody(int i, HashSet<string> stop)
        {
            var children = new JsonArray();

            while (i < source.Length)
            {
                if (StartsAt(i, "<%") && stop.Count > 0)
                {
                    var block = ReadCodeBlock(i);
                    var head = CodeHead.Classify(block.Content, block.ContentBase, block.Tokens);
                    if (stop.Contains(head.KindName)) return (children, i);
                }

                if (StartsAt(i, "<#"))
                {
                    var end = FindDelimiter(i + 2, "#>");
                    if (end < 0)
                    {
                        throw new ParseError(
                            "unclosed comment",
                            i,
                            2,
                            file,
                            "comment starts here");
                    }
                    children.Add(new JsonObject
                    {
                        ["type"] = "Comment",
                        ["value"] = source[(i + 2)..end],
                    });
                    i = end + 2;
                    continue;
                }

                if (StartsAt(i, "<%!"))
                {
                    children.Add(new JsonObject
                    {
                        ["type"] = "Text",
                        ["value"] = "<%",
                    });
                    i += 3;
                    continue;
                }

                if (StartsAt(i, "<%"))
                {
                    var open = i;
                    var block = ReadCodeBlock(i);
                    i = block.Next;
                    var head = CodeHead.Classify(block.Content, block.ContentBase, block.Tokens);

                    if (stop.Count > 0 && stop.Contains(head.KindName))
                        return (children, open);

                    switch (head)
                    {
                        case CodeHead.If ifHead:
                        {
                            var (node, ni) = ParseIf(i, ifHead, block.Trim);
                            children.Add(node);
                            i = ni;
                            break;
                        }
                        case CodeHead.Loop loopHead:
                        {
                            var (node, ni) = ParseLoop(i, loopHead, block.Trim);
                            children.Add(node);
                            i = ni;
                            break;
                        }
                        case CodeHead.EndIf or CodeHead.EndLoop or CodeHead.Else or CodeHead.ElseIf:
                            if (stop.Count == 0)
                            {
                                throw new ParseError(
                                    $"unexpected control `{head.KindName}`",
                                    block.ContentBase,
                                    1,
                                    file);
                            }
                            return (children, open);
                        case CodeHead.Expr exprHead:
                            children.Add(new JsonObject
                            {
                                ["type"] = "Interpolation",
                                ["expression"] = ExprParser.Parse(
                                    exprHead.ExprSlice,
                                    source,
                                    file,
                                    exprHead.ExprAbs),
                                ["trim"] = block.Trim,
                            });
                            break;
                    }
                    continue;
                }

                var next = NextMarkup(i);
                if (next is null)
                {
                    children.Add(new JsonObject
                    {
                        ["type"] = "Text",
                        ["value"] = source[i..],
                    });
                    break;
                }
                if (next > i)
                {
                    children.Add(new JsonObject
                    {
                        ["type"] = "Text",
                        ["value"] = source[i..next.Value],
                    });
                    i = next.Value;
                }
                else i++;
            }

            return (children, i);
        }

        (JsonObject node, int i) ParseIf(int i, CodeHead.If head, string trim)
        {
            var test = ExprParser.Parse(head.TestSlice, source, file, head.TestAbs);
            var stop = new HashSet<string> { "else_if", "else", "end_if" };
            var (consequent, pos) = ParseBody(i, stop);
            i = pos;
            var elseIfs = new JsonArray();
            JsonArray? alternate = null;

            while (true)
            {
                var block = ReadCodeBlock(i);
                var h = CodeHead.Classify(block.Content, block.ContentBase, block.Tokens);
                switch (h)
                {
                    case CodeHead.ElseIf elseIf:
                    {
                        i = block.Next;
                        var t = ExprParser.Parse(elseIf.TestSlice, source, file, elseIf.TestAbs);
                        var (body, n) = ParseBody(i, stop);
                        elseIfs.Add(new JsonObject
                        {
                            ["type"] = "Stmt.ElseIf",
                            ["test"] = t,
                            ["consequent"] = body,
                            ["trim"] = "none",
                        });
                        i = n;
                        break;
                    }
                    case CodeHead.Else:
                    {
                        i = block.Next;
                        var (body, n) = ParseBody(i, new HashSet<string> { "end_if" });
                        alternate = body;
                        i = n;
                        var end = ReadCodeBlock(i);
                        if (CodeHead.Classify(end.Content, end.ContentBase, end.Tokens) is not CodeHead.EndIf)
                        {
                            throw new ParseError("expected `end if`", end.ContentBase, 1, file);
                        }
                        i = end.Next;
                        goto Done;
                    }
                    case CodeHead.EndIf:
                        i = block.Next;
                        goto Done;
                    default:
                        throw new ParseError(
                            $"expected if closer, got `{h.KindName}`",
                            block.ContentBase,
                            1,
                            file);
                }
            }

        Done:
            var node = new JsonObject
            {
                ["type"] = "Stmt.If",
                ["test"] = test,
                ["consequent"] = consequent,
                ["elseIfs"] = elseIfs,
                ["trim"] = trim,
            };
            if (alternate is not null) node["alternate"] = alternate;
            return (node, i);
        }

        (JsonObject node, int i) ParseLoop(int i, CodeHead.Loop head, string trim)
        {
            var iterable = ExprParser.Parse(head.IterSlice, source, file, head.IterAbs);
            var (body, pos) = ParseBody(i, new HashSet<string> { "end_loop" });
            i = pos;
            var end = ReadCodeBlock(i);
            if (CodeHead.Classify(end.Content, end.ContentBase, end.Tokens) is not CodeHead.EndLoop)
            {
                throw new ParseError("expected `end loop`", end.ContentBase, 1, file);
            }
            return (new JsonObject
            {
                ["type"] = "Stmt.For",
                ["item"] = head.Item,
                ["iterable"] = iterable,
                ["body"] = body,
                ["trim"] = trim,
            }, end.Next);
        }

        CodeBlock ReadCodeBlock(int i)
        {
            if (!StartsAt(i, "<%"))
                throw new ParseError("expected code open `<%`", i, 1, file);

            var j = i + 2;
            var trim = "none";
            if (j < source.Length)
            {
                var mod = source[j];
                if (mod is '.' or '_' or '-' or '~' or '=')
                {
                    trim = mod switch
                    {
                        '_' => "ws",
                        '-' => "nl",
                        '~' => "ws_nl",
                        '=' => "all",
                        _ => "none",
                    };
                    j++;
                }
            }

            var end = FindDelimiter(j, "%>");
            if (end < 0)
            {
                throw new ParseError("unclosed code block", i, 2, file, "opens here");
            }

            var contentBase = j;
            while (contentBase < end)
            {
                var c = source[contentBase];
                if (c is ' ' or '\t' or '\n' or '\r') contentBase++;
                else break;
            }
            var contentEnd = end;
            while (contentEnd > contentBase)
            {
                var c = source[contentEnd - 1];
                if (c is ' ' or '\t' or '\n' or '\r') contentEnd--;
                else break;
            }

            var content = source[contentBase..contentEnd];
            var tokens = CodeLexer.Lex(content, source, file, contentBase)
                .Where(t => t.Kind != CodeTokenKind.CodeEnd)
                .ToList();
            return new CodeBlock(tokens, content, contentBase, trim, end + 2);
        }

        int FindDelimiter(int from, string delim)
        {
            var d0 = delim[0];
            for (var i = from; i + delim.Length <= source.Length; i++)
            {
                if (source[i] != d0) continue;
                var ok = true;
                for (var k = 1; k < delim.Length; k++)
                {
                    if (source[i + k] != delim[k])
                    {
                        ok = false;
                        break;
                    }
                }
                if (ok) return i;
            }
            return -1;
        }

        int? NextMarkup(int from)
        {
            for (var i = from; i + 1 < source.Length; i++)
            {
                if (source[i] != '<') continue;
                var n = source[i + 1];
                if (n is '%' or '#') return i;
            }
            return null;
        }

        bool StartsAt(int i, string s)
        {
            if (i + s.Length > source.Length) return false;
            for (var k = 0; k < s.Length; k++)
            {
                if (source[i + k] != s[k]) return false;
            }
            return true;
        }

        readonly record struct CodeBlock(
            List<CodeToken> Tokens,
            string Content,
            int ContentBase,
            string Trim,
            int Next);
    }
}
