namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     LSP Hover 对象
/// </summary>
public sealed class LspHover
{
    /// <summary>
    ///     悬停范围
    /// </summary>
    public LspRange range { get; init; } = new();


    /// <summary>
    ///     悬停内容（Markdown 格式）
    /// </summary>
    public string contents { get; init; } = string.Empty;
}