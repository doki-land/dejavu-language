using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.Json.Serialization;

namespace Dejavu.Types;

public static class IrDefaults
{
    public static readonly JsonObject DefaultLanguage = new()
    {
        ["syntaxMode"] = "template",
        ["template"] = new JsonObject
        {
            ["codeStart"] = "<%",
            ["codeEnd"] = "%>",
            ["commentStart"] = "<#",
            ["commentEnd"] = "#>",
            ["supportFilterPipe"] = true,
            ["legacyFor"] = false,
        },
    };
}

public static class IrNormalize
{
    public static JsonNode? Normalize(JsonNode? node)
    {
        if (node is null) return null;
        if (node is JsonArray arr)
        {
            var outArr = new JsonArray();
            foreach (var item in arr)
            {
                var n = Normalize(item?.DeepClone());
                if (n is not null) outArr.Add(n);
            }
            return outArr;
        }

        if (node is JsonObject obj)
        {
            var type = obj["type"]?.GetValue<string>();
            if (type == "Text" && obj["value"]?.GetValue<string>() == "")
                return null;

            var keys = obj.Select(p => p.Key).Where(k => k != "span").OrderBy(k => k, StringComparer.Ordinal).ToList();
            var result = new JsonObject();
            foreach (var key in keys)
            {
                if (key == "raw" && obj[key] is JsonValue jv && jv.TryGetValue<bool>(out var b) && !b)
                    continue;
                result[key] = Normalize(obj[key]?.DeepClone());
            }
            return result;
        }

        return node.DeepClone();
    }

    public static string NormalizeJson(string json)
    {
        var node = JsonNode.Parse(json);
        return Normalize(node)!.ToJsonString(new JsonSerializerOptions { WriteIndented = false });
    }
}

public static class IrFilters
{
    public static JsonNode? Apply(string name, JsonNode? value, IReadOnlyList<JsonNode?> args)
    {
        return name switch
        {
            "uppercase" => JsonValue.Create(ToString(value).ToUpperInvariant()),
            "lowercase" => JsonValue.Create(ToString(value).ToLowerInvariant()),
            "trim" => JsonValue.Create(ToString(value).Trim()),
            "default" => value is null || (value is JsonValue v && v.TryGetValue<string>(out var s) && s.Length == 0)
                ? args.FirstOrDefault()
                : value,
            "length" => JsonValue.Create(Length(value)),
            "join" => JsonValue.Create(Join(value, args.FirstOrDefault())),
            "replace" => JsonValue.Create(ToString(value).Replace(ToString(args.ElementAtOrDefault(0)), ToString(args.ElementAtOrDefault(1)))),
            _ => throw new InvalidOperationException($"unknown filter `{name}`"),
        };
    }

    public static string ToString(JsonNode? v) => v switch
    {
        null => "",
        JsonValue jv when jv.TryGetValue<string>(out var s) => s,
        JsonValue jv when jv.TryGetValue<bool>(out var b) => b ? "true" : "false",
        JsonValue => v.ToJsonString(),
        _ => v.ToJsonString(),
    };

    static int Length(JsonNode? v) => v switch
    {
        JsonArray a => a.Count,
        JsonObject o => o.Count,
        JsonValue jv when jv.TryGetValue<string>(out var s) => s.Length,
        _ => 0,
    };

    static string Join(JsonNode? v, JsonNode? sepNode)
    {
        var sep = sepNode is null ? "," : ToString(sepNode);
        if (v is JsonArray a)
            return string.Join(sep, a.Select(ToString));
        return ToString(v);
    }
}
