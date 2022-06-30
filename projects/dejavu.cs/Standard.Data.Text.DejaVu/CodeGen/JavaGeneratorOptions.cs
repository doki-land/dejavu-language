namespace Std.Data.Text.DejaVu.CodeGen;

/// <summary>
///     Java 代码生成选项
/// </summary>
public sealed class JavaGeneratorOptions
{
    /// <summary>
    ///     是否包含文件头注释
    /// </summary>
    public bool include_header { get; init; } = true;


    /// <summary>
    ///     是否内联辅助方法
    /// </summary>
    public bool emit_helpers_inline { get; init; } = true;


    /// <summary>
    ///     缩进空格数
    /// </summary>
    public int indent_size { get; init; } = 4;
}