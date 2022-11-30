namespace Std.Data.Text.DejaVu.Debug;

/// <summary>
///     性能剖析条目
/// </summary>
public sealed class ProfileEntry
{
    /// <summary>
    ///     节点类型
    /// </summary>
    public string node_type { get; init; } = string.Empty;


    /// <summary>
    ///     执行次数
    /// </summary>
    public int count { get; init; }


    /// <summary>
    ///     总耗时（毫秒）
    /// </summary>
    public double total_ms { get; init; }


    /// <summary>
    ///     平均耗时（毫秒）
    /// </summary>
    public double avg_ms { get; init; }


    /// <summary>
    ///     最大耗时（毫秒）
    /// </summary>
    public double max_ms { get; init; }
}