namespace Std.Data.Text.DejaVu.Macros;

/// <summary>
///     宏节点
/// </summary>
public sealed class MacroNode : IMacroNode
{
    public string name { get; init; } = string.Empty;
    public Dictionary<string, object> arguments { get; init; } = new();
    public string type => "macro";
}