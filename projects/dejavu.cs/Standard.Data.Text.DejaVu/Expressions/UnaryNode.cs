namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     一元节点
/// </summary>
public sealed class UnaryNode : IExpressionNode
{
    /// <summary>
    ///     运算符
    /// </summary>
    public UnaryOperator @operator { get; init; }


    /// <summary>
    ///     操作数
    /// </summary>
    public IExpressionNode operand { get; init; } = null!;
}