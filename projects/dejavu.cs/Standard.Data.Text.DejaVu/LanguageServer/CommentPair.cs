namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     注释对
/// </summary>
public sealed class CommentPair
{
    /// <summary>
    ///     开始标记
    /// </summary>
    public string open { get; init; } = string.Empty;


    /// <summary>
    ///     结束标记
    /// </summary>
    public string close { get; init; } = string.Empty;
}