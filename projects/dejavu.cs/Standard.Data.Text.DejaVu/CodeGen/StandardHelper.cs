using Std.Data.Text.DejaVu.Optimizer;

namespace Std.Data.Text.DejaVu.CodeGen;

/// <summary>
///     标准辅助函数定义
/// </summary>
public sealed class StandardHelper
{
    /// <summary>
    ///     创建标准辅助函数定义
    /// </summary>
    public StandardHelper(string name, string description, params HelperParameter[] parameters)
    {
        this.name = name;
        this.description = description;
        this.parameters = parameters;
    }

    /// <summary>
    ///     函数名
    /// </summary>
    public string name { get; }


    /// <summary>
    ///     函数描述
    /// </summary>
    public string description { get; }


    /// <summary>
    ///     参数列表
    /// </summary>
    public HelperParameter[] parameters { get; }


    /// <summary>
    ///     输入类型
    /// </summary>
    public TemplateType input_type { get; init; } = TemplateType.any;


    /// <summary>
    ///     输出类型
    /// </summary>
    public TemplateType output_type { get; init; } = TemplateType.any;


    /// <summary>
    ///     TypeScript 实现
    /// </summary>
    public string ts_implementation { get; init; } = string.Empty;


    /// <summary>
    ///     Java 实现
    /// </summary>
    public string java_implementation { get; init; } = string.Empty;
}