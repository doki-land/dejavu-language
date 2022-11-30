using Std.Data.Text.DejaVu.Optimizer;

namespace Std.Data.Text.DejaVu.CodeGen;

/// <summary>
///     辅助函数参数定义
/// </summary>
public sealed class HelperParameter
{
    /// <summary>
    ///     创建辅助函数参数定义
    /// </summary>
    public HelperParameter(string name, TemplateType type, string description)
    {
        this.name = name;
        this.type = type;
        this.description = description;
    }

    /// <summary>
    ///     参数名
    /// </summary>
    public string name { get; }


    /// <summary>
    ///     参数类型
    /// </summary>
    public TemplateType type { get; }


    /// <summary>
    ///     参数描述
    /// </summary>
    public string description { get; }
}