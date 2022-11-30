namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     LSP Position 对象
/// </summary>
public sealed class LspPosition
{
    /// <summary>
    ///     行号（0-based）
    /// </summary>
    public int line { get; init; }


    /// <summary>
    ///     列号（0-based，UTF-16 代码单元）
    /// </summary>
    public int character { get; init; }
}