namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     缩进规则
/// </summary>
public sealed class IndentationRules
{
    /// <summary>
    ///     增加缩进模式
    /// </summary>
    public string increase_indent_pattern { get; init; } = string.Empty;


    /// <summary>
    ///     减少缩进模式
    /// </summary>
    public string decrease_indent_pattern { get; init; } = string.Empty;
}