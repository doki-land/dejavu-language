namespace Std.Data.Text.DejaVu.Macros;

/// <summary>
///     宏定义
/// </summary>
public sealed class MacroDefinition
{
    /// <summary>
    ///     宏名称
    /// </summary>
    public string name { get; init; } = string.Empty;


    /// <summary>
    ///     参数列表
    /// </summary>
    public IReadOnlyList<string> parameters { get; init; } = new List<string>();


    /// <summary>
    ///     宏体
    /// </summary>
    public IReadOnlyList<IMacroNode> body { get; init; } = new List<IMacroNode>();


    /// <summary>
    ///     展开宏
    /// </summary>
    public List<MacroNode> expand(Dictionary<string, object> arguments)
    {
        var result = new List<MacroNode>();
        foreach (var node in body) result.AddRange(expand_node(node, arguments));

        return result;
    }


    /// <summary>
    ///     展开节点
    /// </summary>
    private List<MacroNode> expand_node(IMacroNode node, Dictionary<string, object> arguments)
    {
        return node switch
        {
            MacroTextNode textNode =>
            [
                new MacroNode { name = "text", arguments = new Dictionary<string, object> { ["text"] = textNode.text } }
            ],
            MacroCodeNode codeNode => expand_code_node(codeNode, arguments),
            MacroNode macroNode => [macroNode],
            _ => []
        };
    }


    /// <summary>
    ///     展开代码节点
    /// </summary>
    private List<MacroNode> expand_code_node(MacroCodeNode codeNode, Dictionary<string, object> arguments)
    {
        // 检查是否是参数引用
        var code = codeNode.code.Trim();
        if (arguments.TryGetValue(code, out var value))
            return
            [
                new MacroNode
                {
                    name = "text", arguments = new Dictionary<string, object> { ["text"] = value?.ToString() ?? "" }
                }
            ];

        return
        [
            new MacroNode { name = "code", arguments = new Dictionary<string, object> { ["code"] = codeNode.code } }
        ];
    }
}