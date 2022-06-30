namespace Std.Data.Text.DejaVu.Macros;

/// <summary>
///     宏注册表
/// </summary>
public sealed class MacroRegistry
{
    private readonly Dictionary<string, MacroDefinition> _macros;

    public MacroRegistry()
    {
        _macros = new Dictionary<string, MacroDefinition>();
    }


    /// <summary>
    ///     注册宏
    /// </summary>
    public void register(string name, MacroDefinition macro)
    {
        _macros[name] = macro;
    }


    /// <summary>
    ///     获取宏
    /// </summary>
    public MacroDefinition? get(string name)
    {
        return _macros.GetValueOrDefault(name);
    }


    /// <summary>
    ///     检查宏是否存在
    /// </summary>
    public bool exists(string name)
    {
        return _macros.ContainsKey(name);
    }


    /// <summary>
    ///     展开宏
    /// </summary>
    public List<MacroNode> expand(string name, Dictionary<string, object> arguments)
    {
        var macro = get(name);
        if (macro == null) throw new KeyNotFoundException($"Macro not found: {name}");

        return macro.expand(arguments);
    }
}