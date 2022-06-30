namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     字面量节点
/// </summary>
public sealed class LiteralNode : IExpressionNode
{
    /// <summary>
    ///     字面量值
    /// </summary>
    public object? value { get; init; }
}