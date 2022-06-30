namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     折叠配置
/// </summary>
public sealed class FoldingConfiguration
{
    /// <summary>
    ///     折叠标记
    /// </summary>
    public FoldingMarkers markers { get; init; } = new();
}