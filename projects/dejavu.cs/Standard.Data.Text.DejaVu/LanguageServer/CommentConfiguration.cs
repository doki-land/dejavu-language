namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     注释配置
/// </summary>
public sealed class CommentConfiguration
{
    /// <summary>
    ///     行注释（DejaVu 无行注释）
    /// </summary>
    public string? line_comment { get; init; }


    /// <summary>
    ///     块注释
    /// </summary>
    public CommentPair block_comment { get; init; } = new();
}