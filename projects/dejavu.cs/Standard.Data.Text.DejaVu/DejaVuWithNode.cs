using Std.Data.Text.DejaVu.Expressions;

namespace Std.Data.Text.DejaVu;

/// <summary>
///     with 节点（作用域别名）
/// </summary>
public sealed class DejaVuWithNode : DejaVuTemplateNode
{
    /// <inheritdoc />
    public override DejaVuNodeType node_type => DejaVuNodeType.with;


    /// <summary>
    ///     别名
    /// </summary>
    public string alias_name { get; init; } = string.Empty;


    /// <summary>
    ///     表达式
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