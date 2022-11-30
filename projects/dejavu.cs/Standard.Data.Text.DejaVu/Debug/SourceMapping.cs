namespace Std.Data.Text.DejaVu.Debug;

/// <summary>
///     源码映射条目
/// </summary>
public sealed class SourceMapping
{
    /// <summary>
    ///     生成代码行号
    /// </summary>
    public int generated_line { get; init; }


    /// <summary>
    ///     生成代码列号
    /// </summary>
    public int generated_column { get; init; }


    /// <summary>
    ///     源码行号
    /// </summary>
    public int source_line { get; init; }


    /// <summary>
    ///     源码列号
    /// </summary>
    public int source_column { get; init; }


    /// <summary>
    ///     源码文件路径
    /// </summary>
    public string source_file { get; init; } = string.Empty;
}