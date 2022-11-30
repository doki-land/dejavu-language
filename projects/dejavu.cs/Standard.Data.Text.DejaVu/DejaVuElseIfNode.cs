using Std.Data.Text.DejaVu.Expressions;

namespace Std.Data.Text.DejaVu;

/// <summary>
///     else if 节点
/// </summary>
public sealed class DejaVuElseIfNode : DejaVuTemplateNode
{
    /// <inheritdoc />
    public override DejaVuNodeType node_type => DejaVuNodeType.@if;


    /// <summary>
    ///     条件表达式原始文本
    /// </summary>
    public string condition { get; init; } = string.Empty;


    /// <summary>
    ///     预解析的条件表达式 AST
    /// </summary>
    public IExpressionNode? parsed_condition { get; init; }


    /// <summary>
    ///     子节点
    /// </summary>
    public List<DejaVuTemplateNode> children { get; init; } = [];
}