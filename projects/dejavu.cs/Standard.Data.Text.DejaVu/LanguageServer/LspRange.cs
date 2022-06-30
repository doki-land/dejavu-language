namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     LSP Range 对象
/// </summary>
public sealed class LspRange
{
    /// <summary>
    ///     起始位置
    /// </summary>
    public LspPosition start { get; init; } = new();


    /// <summary>
    ///     结束位置
    /// </summary>
    public LspPosition end { get; init; } = new();
}