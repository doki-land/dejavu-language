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
            "Expr.Call" => EvalCall(expr, scope),
            "Expr.Test" => EvalTest(expr, scope),
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

    const int RangeLimit = 10_000;

    static JsonNode? EvalCall(JsonNode expr, Dictionary<string, JsonNode?> scope)
    {
        var callee = expr["callee"]!;
        if (callee["type"]?.GetValue<string>() != "Expr.Identifier")
            throw new InvalidOperationException("only built-in function calls are supported");
        var name = callee["name"]!.GetValue<string>();
        var args = expr["arguments"]!.AsArray().Select(a => Eval(a!, scope)).ToList();
        return name switch
        {
            "range" => RangeCall(args),
            "min" => NumericPair(args, min: true),
            "max" => NumericPair(args, min: false),
            _ => throw new InvalidOperationException($"dejavu.template.unknown-function: {name}"),
        };
    }

    static JsonNode? RangeCall(List<JsonNode?> args)
    {
        if (args.Count is < 2 or > 3)
            throw new InvalidOperationException($"dejavu.template.function-arity: range got {args.Count}");
        if (!Integral(args[0], out var start) || !Integral(args[1], out var stop))
            return null;
        long step = 1;
        if (args.Count == 3 && (!Integral(args[2], out step) || step == 0))
            return null;
        var outArray = new JsonArray();
        for (var current = start; (step > 0 && current < stop) || (step < 0 && current > stop); current += step)
        {
            if (outArray.Count >= RangeLimit) return null;
            outArray.Add(JsonValue.Create(current));
        }
        return outArray;
    }

    static JsonNode? NumericPair(List<JsonNode?> args, bool min)
    {
        if (args.Count != 2)
            throw new InvalidOperationException("dejavu.template.function-arity: min/max expect 2 arguments");
        if (!FiniteNumber(args[0], out var a) || !FiniteNumber(args[1], out var b))
            return null;
        var picked = min ? Math.Min(a, b) : Math.Max(a, b);
        // 整数值保持整数形态，字符串与 Rust/TS 的 `4` 而非 `4.0` 对齐。
        return picked == Math.Floor(picked) && picked >= long.MinValue && picked <= long.MaxValue
            ? JsonValue.Create((long)picked)
            : JsonValue.Create(picked);
    }

    static JsonNode? EvalTest(JsonNode expr, Dictionary<string, JsonNode?> scope)
    {
        var name = expr["test"]!.GetValue<string>();
        var value = Eval(expr["expression"]!, scope);
        var args = expr["arguments"]?.AsArray().Select(a => Eval(a!, scope)).ToList() ?? [];
        if (args.Count != 0)
            throw new InvalidOperationException($"dejavu.template.test-arity: {name} got {args.Count}");
        var negated = expr["negated"]?.GetValue<bool>() == true;
        var result = name switch
        {
            "null" or "none" => value is null,
            "truthy" => Truthy(value),
            "empty" => value is JsonValue sv && sv.TryGetValue<string>(out var s)
                ? s.Length == 0
                : value is JsonArray ea ? ea.Count == 0 : value is JsonObject eo && eo.Count == 0,
            "array" => value is JsonArray,
            "object" => value is JsonObject,
            "string" => value is JsonValue sv2 && sv2.TryGetValue<string>(out _),
            "number" => FiniteNumber(value, out _),
            "boolean" => value is JsonValue bv && bv.TryGetValue<bool>(out _),
            "even" => Integral(value, out var evenN) && evenN % 2 == 0,
            "odd" => Integral(value, out var oddN) && oddN % 2 != 0,
            _ => throw new InvalidOperationException($"dejavu.template.unknown-test: {name}"),
        };
        return JsonValue.Create(result != negated);
    }

    static bool Integral(JsonNode? value, out long result)
    {
        result = 0;
        if (!FiniteNumber(value, out var d) || d != Math.Floor(d)) return false;
        if (d < long.MinValue || d > long.MaxValue) return false;
        result = (long)d;
        return true;
    }

    static bool FiniteNumber(JsonNode? value, out double result)
    {
        result = 0;
        return value is JsonValue jv
            && jv.TryGetValue<double>(out result)
            && double.IsFinite(result);
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
        // 比较只对 number 成立，其他类型不转型，直接 false（与 Rust/TS 一致）。
        "<" or "<=" or ">" or ">=" => JsonValue.Create(false),
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
        if (r is JsonObject o)
            return o.ContainsKey(IrFilters.ToString(l));
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
