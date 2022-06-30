namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     标识符节点
/// </summary>
public sealed class IdentifierNode : IExpressionNode
{
    /// <summary>
    ///     标识符名称
    /// </summary>
    public string name { get; init; } = string.Empty;
}