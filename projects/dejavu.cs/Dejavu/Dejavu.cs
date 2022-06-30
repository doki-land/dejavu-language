using System.Text.Json.Nodes;

namespace Dejavu;

/// <summary>
/// Public .NET entry point for the Dejavu template engine.
/// </summary>
/// <remarks>
/// <para>
/// Install / reference the <b>Dejavu</b> package only. Internal projects
/// (<c>Dejavu.Language</c>, <c>Dejavu.Types</c>, <c>Dejavu.Engine</c>) are not
/// part of the supported application surface.
/// </para>
/// <para>
/// Because the package namespace is also <c>Dejavu</c>, call this type as
/// <c>Dejavu.Dejavu</c>, or alias it:
/// <code>
/// using Dj = Dejavu.Dejavu;
/// var output = Dj.RenderSource("Hello, &lt;% name %&gt;!", new JsonObject { ["name"] = "World" });
/// </code>
/// This matches the <c>Dejavu</c> facade in TypeScript, Python, Rust, and Kotlin.
/// </para>
/// </remarks>
public static class Dejavu
{
    /// <summary>Parse template source → Dejavu IR document.</summary>
    public static JsonObject Parse(string source)
        => Language.T1Parser.ParseToIr(source);

    /// <summary>
    /// Render IR + context → string.
    /// Same IR + context must produce byte-identical output across host languages.
    /// </summary>
    public static string Render(JsonObject ir, JsonObject? ctx = null)
        => Engine.IrRenderer.Render(ir, ctx ?? new JsonObject());

    /// <summary>Parse then render.</summary>
    public static string RenderSource(string source, JsonObject? ctx = null)
        => Engine.IrRenderer.RenderSource(source, ctx ?? new JsonObject());

    /// <summary>Syntax check (parse only).</summary>
    public static (bool Valid, string[] Errors) Check(string source)
    {
        try
        {
            _ = Parse(source);
            return (true, Array.Empty<string>());
        }
        catch (Exception ex)
        {
            return (false, new[] { ex.Message });
        }
    }

    /// <summary>Normalize IR JSON for semantic equality.</summary>
    public static string NormalizeIrJson(string json)
        => Types.IrNormalize.NormalizeJson(json);
}

/// <summary>
/// Compatibility alias for older call sites. Prefer <see cref="Dejavu"/>.
/// </summary>
public static class DejavuEngine
{
    /// <inheritdoc cref="Dejavu.Parse"/>
    public static JsonObject Parse(string source) => Dejavu.Parse(source);

    /// <inheritdoc cref="Dejavu.Render"/>
    public static string Render(JsonObject ir, JsonObject? ctx = null) => Dejavu.Render(ir, ctx);

    /// <inheritdoc cref="Dejavu.RenderSource"/>
    public static string RenderSource(string source, JsonObject? ctx = null)
        => Dejavu.RenderSource(source, ctx);

    /// <inheritdoc cref="Dejavu.Check"/>
    public static (bool Valid, string[] Errors) Check(string source) => Dejavu.Check(source);

    /// <inheritdoc cref="Dejavu.NormalizeIrJson"/>
    public static string NormalizeIrJson(string json) => Dejavu.NormalizeIrJson(json);
}
