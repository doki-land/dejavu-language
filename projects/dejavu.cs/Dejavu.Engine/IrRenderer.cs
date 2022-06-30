using System.Text;
using System.Text.Json.Nodes;
using Dejavu.Language;
using Dejavu.Types;

namespace Dejavu.Engine;

public static class IrRenderer
{
    public static string Render(JsonObject doc, JsonObject ctx)
    {
        var scope = ctx.ToDictionary(p => p.Key, p => p.Value?.DeepClone());
        return RenderNode(doc["body"]!, scope);
    }

    public static string RenderSource(string source, JsonObject ctx)
        => Render(T1Parser.ParseToIr(source), ctx);

    static string RenderNode(JsonNode node, Dictionary<string, JsonNode?> scope)
    {
        var type = node["type"]!.GetValue<string>();
        return type switch
        {
            "Template" => JoinChildren(node["children"]!.AsArray(), scope),
            "Text" => node["value"]!.GetValue<string>(),
            "Comment" => "",
            "Interpolation" => Format(Eval(node["expression"]!, scope), node["raw"]?.GetValue<bool>() == true),
            "Stmt.If" => RenderIf(node, scope),
            "Stmt.For" => RenderFor(node, scope),
            "Stmt.Raw" => node["value"]!.GetValue<string>(),
            "Stmt.Block" => JoinChildren(node["body"]!.AsArray(), scope),
            "Stmt.Extends" or "Stmt.Include" or "Stmt.Super"
                => throw new InvalidOperationException("extends/include/super need a loader"),
            _ => throw new InvalidOperationException($"node not renderable: {type}"),
        };
    }

    static string RenderIf(JsonNode node, Dictionary<string, JsonNode?> scope)
    {
        if (Truthy(Eval(node["test"]!, scope)))
            return JoinChildren(node["consequent"]!.AsArray(), scope);
        foreach (var ei in node["elseIfs"]!.AsArray())
        {
            if (Truthy(Eval(ei!["test"]!, scope)))
                return JoinChildren(ei["consequent"]!.AsArray(), scope);
        }
        if (node["alternate"] is JsonArray alt)
            return JoinChildren(alt, scope);
        return "";
    }

    static string RenderFor(JsonNode node, Dictionary<string, JsonNode?> scope)
    {
        var item = node["item"]!.GetValue<string>();
        var iterable = Eval(node["iterable"]!, scope) as JsonArray
            ?? throw new InvalidOperationException("for iterable must be array");
        var sb = new StringBuilder();
        var indexName = node["index"]?.GetValue<string>();
        for (var i = 0; i < iterable.Count; i++)
        {
            scope.TryGetValue(item, out var prev);
            scope[item] = iterable[i]?.DeepClone();
            JsonNode? prevIdx = null;
            if (indexName is not null)
            {
                scope.TryGetValue(indexName, out prevIdx);
                scope[indexName] = JsonValue.Create(i);
            }
            sb.Append(JoinChildren(node["body"]!.AsArray(), scope));
            if (prev is null) scope.Remove(item); else scope[item] = prev;
            if (indexName is not null)
            {
                if (prevIdx is null) scope.Remove(indexName); else scope[indexName] = prevIdx;
            }
        }
        return sb.ToString();
    }

    static string JoinChildren(JsonArray children, Dictionary<string, JsonNode?> scope)
    {
        var sb = new StringBuilder();
        foreach (var c in children) sb.Append(RenderNode(c!, scope));
        return sb.ToString();
    }

    static string Format(JsonNode? v, bool raw)
    {
        var s = IrFilters.ToString(v);
        return raw ? s : HtmlEscape(s);
    }

    static string HtmlEscape(string s) =>
        s.Replace("&", "&amp;").Replace("<", "&lt;").Replace(">", "&gt;")
            .Replace("\"", "&quot;").Replace("'", "&#39;");

    static JsonNode? Eval(JsonNode expr, Dictionary<string, JsonNode?> scope)
    {
        var type = expr["type"]!.GetValue<string>();
        return type switch
        {
            "Expr.Literal" => expr["value"]?.DeepClone(),
            "Expr.Identifier" => scope.TryGetValue(expr["name"]!.GetValue<string>(), out var v) ? v?.DeepClone() : null,
            "Expr.Member" => EvalMember(expr, scope),
            "Expr.Index" => EvalIndex(expr, scope),
            "Expr.Binary" => EvalBinary(expr["operator"]!.GetValue<string>(), Eval(expr["left"]!, scope), Eval(expr["right"]!, scope)),
            "Expr.Unary" => EvalUnary(expr["operator"]!.GetValue<string>(), Eval(expr["argument"]!, scope)),
            "Expr.Pipe" => EvalPipe(expr, scope),
            "Expr.Call" => throw new InvalidOperationException("calls not supported in T1 eval"),
            _ => throw new InvalidOperationException($"invalid expression: {type}"),
        };
    }

    static JsonNode? EvalMember(JsonNode expr, Dictionary<string, JsonNode?> scope)
    {
        var obj = Eval(expr["object"]!, scope) as JsonObject;
        var prop = expr["property"]!.GetValue<string>();
        return obj is not null && obj.TryGetPropertyValue(prop, out var v) ? v?.DeepClone() : null;
    }

    static JsonNode? EvalIndex(JsonNode expr, Dictionary<string, JsonNode?> scope)
    {
        var obj = Eval(expr["object"]!, scope);
        var idx = Eval(expr["index"]!, scope);
        if (obj is JsonArray a && idx is JsonValue jv && jv.TryGetValue<int>(out var i))
            return i >= 0 && i < a.Count ? a[i]?.DeepClone() : null;
        if (obj is JsonObject o && idx is JsonValue ks && ks.TryGetValue<string>(out var key))
            return o.TryGetPropertyValue(key, out var v) ? v?.DeepClone() : null;
        return null;
    }

    static JsonNode? EvalPipe(JsonNode expr, Dictionary<string, JsonNode?> scope)
    {
        var val = Eval(expr["expression"]!, scope);
        var args = expr["arguments"]!.AsArray().Select(a => Eval(a!, scope)).ToList();
        return IrFilters.Apply(expr["filter"]!.GetValue<string>(), val, args);
    }

    static JsonNode? EvalUnary(string op, JsonNode? v) => op switch
    {
        "!" => JsonValue.Create(!Truthy(v)),
        "-" when v is JsonValue n && n.TryGetValue<double>(out var d) => JsonValue.Create(-d),
        "+" => v,
        _ => null,
    };

    static JsonNode? EvalBinary(string op, JsonNode? l, JsonNode? r) => op switch
    {
        "+" when AsDouble(l, out var a) && AsDouble(r, out var b) => JsonValue.Create(a + b),
        "+" => JsonValue.Create(IrFilters.ToString(l) + IrFilters.ToString(r)),
        "-" when AsDouble(l, out var a) && AsDouble(r, out var b) => JsonValue.Create(a - b),
        "*" when AsDouble(l, out var a) && AsDouble(r, out var b) => JsonValue.Create(a * b),
        "/" when AsDouble(l, out var a) && AsDouble(r, out var b) => JsonValue.Create(a / b),
        "%" when AsDouble(l, out var a) && AsDouble(r, out var b) => JsonValue.Create(a % b),
        "==" => JsonValue.Create(JsonNode.DeepEquals(l, r)),
        "!=" => JsonValue.Create(!JsonNode.DeepEquals(l, r)),
        "<" when AsDouble(l, out var a) && AsDouble(r, out var b) => JsonValue.Create(a < b),
        "<=" when AsDouble(l, out var a) && AsDouble(r, out var b) => JsonValue.Create(a <= b),
        ">" when AsDouble(l, out var a) && AsDouble(r, out var b) => JsonValue.Create(a > b),
        ">=" when AsDouble(l, out var a) && AsDouble(r, out var b) => JsonValue.Create(a >= b),
        "&&" => JsonValue.Create(Truthy(l) && Truthy(r)),
        "||" => JsonValue.Create(Truthy(l) || Truthy(r)),
        "in" => JsonValue.Create(InOp(l, r)),
        _ => null,
    };

    static bool InOp(JsonNode? l, JsonNode? r)
    {
        if (r is JsonArray a) return a.Any(x => JsonNode.DeepEquals(x, l));
        if (r is JsonValue sv && sv.TryGetValue<string>(out var s))
            return s.Contains(IrFilters.ToString(l));
        return false;
    }

    static bool AsDouble(JsonNode? n, out double d)
    {
        d = 0;
        return n is JsonValue jv && jv.TryGetValue(out d);
    }

    static bool Truthy(JsonNode? v) => v switch
    {
        null => false,
        JsonValue jv when jv.TryGetValue<bool>(out var b) => b,
        JsonValue jv when jv.TryGetValue<double>(out var d) => d != 0,
        JsonValue jv when jv.TryGetValue<string>(out var s) => s.Length > 0,
        JsonArray a => a.Count > 0,
        JsonObject o => o.Count > 0,
        _ => true,
    };
}

/// <summary>Internal engine handle. Applications should use the public <c>Dejavu</c> package.</summary>
public sealed class EngineHandle
{
    public JsonObject Parse(string source) => T1Parser.ParseToIr(source);
    public string Render(JsonObject ir, JsonObject ctx) => IrRenderer.Render(ir, ctx);
    public string RenderSource(string source, JsonObject ctx) => IrRenderer.RenderSource(source, ctx);
}
