namespace Std.Data.Text.DejaVu.Benchmark;

/// <summary>
///     基准条目
/// </summary>
public sealed class BenchmarkEntry
{
    /// <summary>
    ///     基准名称
    /// </summary>
    public string name { get; init; } = string.Empty;


    /// <summary>
    ///     迭代次数
    /// </summary>
    public int iterations { get; init; }


    /// <summary>
    ///     平均耗时（毫秒）
    /// </summary>
    public double avg_ms { get; init; }


    /// <summary>
    ///     最小耗时（毫秒）
    /// </summary>
    public double min_ms { get; init; }


    /// <summary>
    ///     最大耗时（毫秒）
    /// </summary>
    public double max_ms { get; init; }


    /// <summary>
    ///     P95 耗时（毫秒）
    /// </summary>
    public double p95_ms { get; init; }


    /// <summary>
    ///     中位数耗时（毫秒）
    /// </summary>
    public double median_ms { get; init; }
}