namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     LSP Location 对象
/// </summary>
public sealed class LspLocation
{
    /// <summary>
    ///     文档 URI
    /// </summary>
    public string uri { get; init; } = string.Empty;


    /// <summary>
    ///     范围
    /// </summary>
    public LspRange range { get; init; } = new();
}