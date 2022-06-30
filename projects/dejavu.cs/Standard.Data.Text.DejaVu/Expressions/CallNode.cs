namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     函数调用节点
/// </summary>
public sealed class CallNode : IExpressionNode
{
    /// <summary>
    ///     被调用的函数
    /// </summary>
    public IExpressionNode function { get; init; } = null!;


    /// <summary>
    ///     实参列表
    /// </summary>
    public List<IExpressionNode> arguments { get; init; } = [];
}