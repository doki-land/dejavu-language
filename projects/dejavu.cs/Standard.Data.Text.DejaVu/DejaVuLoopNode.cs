using Std.Data.Text.DejaVu.Expressions;

namespace Std.Data.Text.DejaVu;

/// <summary>
///     loop 节点
/// </summary>
public sealed class DejaVuLoopNode : DejaVuTemplateNode
{
    /// <inheritdoc />
    public override DejaVuNodeType node_type => DejaVuNodeType.loop;


    /// <summary>
    ///     循环表达式（如 "items" 或 "items" 当用 loop in 语法时）
    /// </summary>
    public string expression { get; init; } = string.Empty;


    /// <summary>
    ///     预解析的表达式 AST
    /// </summary>
    public IExpressionNode? parsed_expression { get; init; }


    /// <summary>
    ///     迭代变量名（loop in 语法时使用，如 "item"）
    /// </summary>
    public string? item_name { get; init; }


    /// <summary>
    ///     子节点
    /// </summary>
    public List<DejaVuTemplateNode> children { get; init; } = [];
}