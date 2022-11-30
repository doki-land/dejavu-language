using Std.Data.Text.DejaVu.Expressions;

namespace Std.Data.Text.DejaVu;

/// <summary>
///     代码节点
/// </summary>
public sealed class DejaVuCodeNode : DejaVuTemplateNode
{
    /// <inheritdoc />
    public override DejaVuNodeType node_type => DejaVuNodeType.code;


    /// <summary>
    ///     代码内容
    /// </summary>
    public string code { get; init; } = string.Empty;


    /// <summary>
    ///     预解析的表达式 AST
    /// </summary>
    public IExpressionNode? parsed_expression { get; init; }
}