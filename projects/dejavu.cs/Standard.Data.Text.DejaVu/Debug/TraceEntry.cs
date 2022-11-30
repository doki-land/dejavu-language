namespace Std.Data.Text.DejaVu.Debug;

/// <summary>
///     追踪条目
/// </summary>
public sealed class TraceEntry
{
    /// <summary>
    ///     节点类型
    /// </summary>
    public string node_type { get; init; } = string.Empty;


    /// <summary>
    ///     源码行号
    /// </summary>
    public int source_line { get; init; }


    /// <summary>
    ///     源码列号
    /// </summary>
    public int source_column { get; init; }


    /// <summary>
    ///     详细信息
    /// </summary>
    public string detail { get; init; } = string.Empty;


    /// <summary>
    ///     耗时（毫秒）
    /// </summary>
    public double elapsed_ms { get; init; }


    /// <summary>
    ///     时间戳（相对于追踪开始的毫秒数）
    /// </summary>
    public double timestamp { get; init; }
}