using Microsoft.Extensions.DependencyInjection;

namespace AspNetCore.Dejavu;

/// <summary>DI helpers for ASP.NET Core hosts.</summary>
public static class DejavuServiceCollectionExtensions
{
    /// <summary>
    /// Marks Dejavu as available in the app. The public API is static
    /// (<c>Dejavu.Dejavu</c>); this registers a marker service for hosts that
    /// want an injectable handle.
    /// </summary>
    public static IServiceCollection AddDejavu(this IServiceCollection services)
    {
        services.AddSingleton<DejavuService>();
        return services;
    }
}

/// <summary>Injectable thin wrapper around the public <c>Dejavu</c> facade.</summary>
public sealed class DejavuService
{
    public System.Text.Json.Nodes.JsonObject Parse(string source)
        => global::Dejavu.Dejavu.Parse(source);

    public string Render(System.Text.Json.Nodes.JsonObject ir, System.Text.Json.Nodes.JsonObject? ctx = null)
        => global::Dejavu.Dejavu.Render(ir, ctx);

    public string RenderSource(string source, System.Text.Json.Nodes.JsonObject? ctx = null)
        => global::Dejavu.Dejavu.RenderSource(source, ctx);
}
