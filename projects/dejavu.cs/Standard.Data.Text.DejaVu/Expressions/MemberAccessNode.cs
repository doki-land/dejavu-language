namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     成员访问节点
/// </summary>
public sealed class MemberAccessNode : IExpressionNode
{
    /// <summary>
    ///     目标对象
    /// </summary>
    public IExpressionNode @object { get; init; } = null!;


    /// <summary>
    ///     成员名称
    /// </summary>
    public string member_name { get; init; } = string.Empty;
}