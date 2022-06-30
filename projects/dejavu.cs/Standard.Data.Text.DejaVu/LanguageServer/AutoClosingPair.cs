namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     自动关闭对
/// </summary>
public sealed class AutoClosingPair
{
    /// <summary>
    ///     开字符
    /// </summary>
    public string open { get; init; } = string.Empty;


    /// <summary>
    ///     闭字符
    /// </summary>
    public string close { get; init; } = string.Empty;
}