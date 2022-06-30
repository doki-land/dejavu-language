namespace Std.Data.Text.DejaVu.CodeGen;

/// <summary>
///     TypeScript 代码生成选项
/// </summary>
public sealed class TypeScriptGeneratorOptions
{
    /// <summary>
    ///     是否包含文件头注释
    /// </summary>
    public bool include_header { get; init; } = true;


    /// <summary>
    ///     是否内联辅助函数（escapeHtml/toBoolean/toIterable/areEqual/applyFilter）
    /// </summary>
    public bool emit_helpers_inline { get; init; } = true;


    /// <summary>
    ///     缩进空格数
    /// </summary>
    public int indent_size { get; init; } = 4;
}