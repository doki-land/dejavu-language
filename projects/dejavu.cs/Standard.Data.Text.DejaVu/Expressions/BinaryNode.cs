namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     二元节点
/// </summary>
public sealed class BinaryNode : IExpressionNode
{
    /// <summary>
    ///     运算符
    /// </summary>
    public BinaryOperator @operator { get; init; }


    /// <summary>
    ///     左操作数
    /// </summary>
    public IExpressionNode left { get; init; } = null!;


    /// <summary>
    ///     右操作数
    /// </summary>
    public IExpressionNode right { get; init; } = null!;
}