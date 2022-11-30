using Std.Data.Text.DejaVu.Expressions;

namespace Std.Data.Text.DejaVu;

/// <summary>
///     match 节点
/// </summary>
public sealed class DejaVuMatchNode : DejaVuTemplateNode
{
    /// <inheritdoc />
    public override DejaVuNodeType node_type => DejaVuNodeType.match;


    /// <summary>
    ///     match 表达式
    /// </summary>
    public string expression { get; init; } = string.Empty;


    /// <summary>
    ///     预解析的表达式 AST
    /// </summary>
    public IExpressionNode? parsed_expression { get; init; }


    /// <summary>
    ///     子节点
    /// </summary>
    public List<DejaVuTemplateNode> children { get; init; } = [];
}