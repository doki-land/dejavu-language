namespace Std.Data.Text.DejaVu.Macros;

/// <summary>
///     宏代码节点
/// </summary>
public sealed class MacroCodeNode : IMacroNode
{
    public string code { get; init; } = string.Empty;
    public string type => "code";
}