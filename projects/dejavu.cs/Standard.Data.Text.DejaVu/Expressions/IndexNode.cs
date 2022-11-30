namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     索引节点
/// </summary>
public sealed class IndexNode : IExpressionNode
{
    /// <summary>
    ///     被索引的对象
    /// </summary>
    public IExpressionNode @object { get; init; } = null!;


    /// <summary>
    ///     索引值
    /// </summary>
    public IExpressionNode index { get; init; } = null!;
}