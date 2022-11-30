namespace Std.Data.Text.DejaVu.Cli;

/// <summary>
///     编译结果
/// </summary>
public sealed class CompileResult
{
    /// <summary>
    ///     创建编译结果
    /// </summary>
    public CompileResult(bool success, string message, string output, string outputPath)
    {
        this.success = success;
        this.message = message;
        this.output = output;
        output_path = outputPath;
    }

    /// <summary>
    ///     是否成功
    /// </summary>
    public bool success { get; }


    /// <summary>
    ///     消息
    /// </summary>
    public string message { get; }


    /// <summary>
    ///     输出内容
    /// </summary>
    public string output { get; }


    /// <summary>
    ///     输出路径
    /// </summary>
    public string output_path { get; }
}