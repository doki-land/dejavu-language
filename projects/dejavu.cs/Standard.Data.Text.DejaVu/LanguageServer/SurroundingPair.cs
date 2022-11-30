namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     环绕对
/// </summary>
public sealed class SurroundingPair
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