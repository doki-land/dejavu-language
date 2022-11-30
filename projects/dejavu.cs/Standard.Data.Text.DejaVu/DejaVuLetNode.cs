using Std.Data.Text.DejaVu.Expressions;

namespace Std.Data.Text.DejaVu;

/// <summary>
///     let 节点（局部变量绑定）
/// </summary>
public sealed class DejaVuLetNode : DejaVuTemplateNode
{
    /// <inheritdoc />
    public override DejaVuNodeType node_type => DejaVuNodeType.let;


    /// <summary>
    ///     变量名
    /// </summary>
    public string variable_name { get; init; } = string.Empty;


    /// <summary>
    ///     值表达式
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