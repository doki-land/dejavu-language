namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     折叠标记
/// </summary>
public sealed class FoldingMarkers
{
    /// <summary>
    ///     折叠开始模式
    /// </summary>
    public string start { get; init; } = string.Empty;


    /// <summary>
    ///     折叠结束模式
    /// </summary>
    public string end { get; init; } = string.Empty;
}