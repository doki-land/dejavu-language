namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     管道节点
/// </summary>
public sealed class PipeNode : IExpressionNode
{
    /// <summary>
    ///     管道左侧表达式
    /// </summary>
    public IExpressionNode left { get; init; } = null!;


    /// <summary>
    ///     过滤器名称
    /// </summary>
    public string filter_name { get; init; } = string.Empty;


    /// <summary>
    ///     过滤器参数
    /// </summary>
    public List<IExpressionNode> arguments { get; init; } = [];
}