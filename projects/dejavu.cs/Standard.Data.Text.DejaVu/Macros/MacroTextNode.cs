namespace Std.Data.Text.DejaVu.Macros;

/// <summary>
///     宏文本节点
/// </summary>
public sealed class MacroTextNode : IMacroNode
{
    public string text { get; init; } = string.Empty;
    public string type => "text";
}