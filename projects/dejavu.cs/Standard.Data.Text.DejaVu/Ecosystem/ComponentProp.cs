namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     组件参数定义
/// </summary>
public sealed class ComponentProp
{
    /// <summary>
    ///     参数名
    /// </summary>
    public string name { get; init; } = string.Empty;


    /// <summary>
    ///     参数类型
    /// </summary>
    public string type { get; init; } = "any";


    /// <summary>
    ///     参数默认值
    /// </summary>
    public string? default_value { get; init; }


    /// <summary>
    ///     参数描述
    /// </summary>
    public string description { get; init; } = string.Empty;


    /// <summary>
    ///     是否必填
    /// </summary>
    public bool required { get; init; }
}