using System.Text.Json.Nodes;
using Dj = Dejavu.Dejavu;

if (args.Length == 0)
{
    Console.WriteLine("Usage: dejavu parse <file> | dejavu render <file> [--from-ir] [--ctx file]");
    return 1;
}

var cmd = args[0];
if (cmd == "parse" && args.Length >= 2)
{
    var ir = Dj.Parse(File.ReadAllText(args[1]));
    Console.WriteLine(ir.ToJsonString(new System.Text.Json.JsonSerializerOptions { WriteIndented = true }));
    return 0;
}

if (cmd == "render" && args.Length >= 2)
{
    var fromIr = args.Contains("--from-ir");
    var ctxPath = ArgsValue(args, "--ctx");
    var ctx = ctxPath is null
        ? new JsonObject()
        : JsonNode.Parse(File.ReadAllText(ctxPath))!.AsObject();
    var raw = File.ReadAllText(args[1]);
    var outText = fromIr
        ? Dj.Render(JsonNode.Parse(raw)!.AsObject(), ctx)
        : Dj.RenderSource(raw, ctx);
    Console.Write(outText);
    return 0;
}

Console.Error.WriteLine("unknown command");
return 1;

static string? ArgsValue(string[] args, string name)
{
    var i = Array.IndexOf(args, name);
    return i >= 0 && i + 1 < args.Length ? args[i + 1] : null;
}
