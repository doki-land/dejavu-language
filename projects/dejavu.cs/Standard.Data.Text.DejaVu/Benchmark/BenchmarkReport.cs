namespace Std.Data.Text.DejaVu.Benchmark;

/// <summary>
///     基准报告
/// </summary>
public sealed class BenchmarkReport
{
    /// <summary>
    ///     运行时间
    /// </summary>
    public DateTimeOffset run_at { get; init; }


    /// <summary>
    ///     运行环境
    /// </summary>
    public string machine_info { get; init; } = string.Empty;


    /// <summary>
    ///     编译基准
    /// </summary>
    public List<BenchmarkEntry> compile_benchmarks { get; init; } = [];


    /// <summary>
    ///     渲染基准
    /// </summary>
    public List<BenchmarkEntry> render_benchmarks { get; init; } = [];


    /// <summary>
    ///     代码生成基准
    /// </summary>
    public List<BenchmarkEntry> code_gen_benchmarks { get; init; } = [];
}