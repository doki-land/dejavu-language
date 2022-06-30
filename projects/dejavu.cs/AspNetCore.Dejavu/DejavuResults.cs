using System.Text.Json.Nodes;
using Microsoft.AspNetCore.Http;
using Dj = Dejavu.Dejavu;

namespace AspNetCore.Dejavu;

/// <summary>
/// Minimal API helpers. Host prefix: <c>AspNetCore.*</c>.
/// Renders through the public <c>Dejavu</c> package (IR path).
/// </summary>
public static class DejavuResults
{
    /// <summary>Parse source → IR → HTML response.</summary>
    public static IResult Html(string source, JsonObject? ctx = null)
    {
        var body = Dj.RenderSource(source, ctx);
        return Results.Content(body, "text/html; charset=utf-8");
    }

    /// <summary>Render IR + context → HTML (cross-language identical path).</summary>
    public static IResult HtmlFromIr(JsonObject ir, JsonObject? ctx = null)
    {
        var body = Dj.Render(ir, ctx);
        return Results.Content(body, "text/html; charset=utf-8");
    }

    /// <summary>Deserialize IR JSON string then render.</summary>
    public static IResult HtmlFromIrJson(string irJson, JsonObject? ctx = null)
    {
        var ir = JsonNode.Parse(irJson)?.AsObject()
            ?? throw new InvalidOperationException("IR JSON must be an object");
        return HtmlFromIr(ir, ctx);
    }
}
