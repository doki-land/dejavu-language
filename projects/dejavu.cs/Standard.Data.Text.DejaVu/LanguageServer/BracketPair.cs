namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     括号对
/// </summary>
public sealed class BracketPair
{
    /// <summary>
    ///     开括号
    /// </summary>
    public string open { get; init; } = string.Empty;


    /// <summary>
    ///     闭括号
    /// </summary>
    public string close { get; init; } = string.Empty;
}