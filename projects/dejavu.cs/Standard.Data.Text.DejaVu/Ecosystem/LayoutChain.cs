namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     布局继承链
/// </summary>
public sealed class LayoutChain
{
    /// <summary>
    ///     创建布局继承链
    /// </summary>
    public LayoutChain(IReadOnlyList<LayoutNode> nodes, LayoutResolveStatus status, string? errorMessage)
    {
        this.nodes = nodes;
        this.status = status;
        error_message = errorMessage;
    }

    /// <summary>
    ///     继承链节点（索引 0 为根布局，最后一个为当前模板）
    /// </summary>
    public IReadOnlyList<LayoutNode> nodes { get; }


    /// <summary>
    ///     解析状态
    /// </summary>
    public LayoutResolveStatus status { get; }


    /// <summary>
    ///     错误消息
    /// </summary>
    public string? error_message { get; }


    /// <summary>
    ///     继承深度
    /// </summary>
    public int depth => nodes.Count;
}