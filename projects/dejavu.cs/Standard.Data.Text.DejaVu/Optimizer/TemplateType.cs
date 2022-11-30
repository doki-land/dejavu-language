namespace Std.Data.Text.DejaVu.Optimizer;

/// <summary>
///     模板类型——编译期类型系统的类型枚举
/// </summary>
public enum TemplateType
{
    /// <summary>
    ///     未知类型（无法推导）
    /// </summary>
    unknown,


    /// <summary>
    ///     任意类型（动态）
    /// </summary>
    any,


    /// <summary>
    ///     空值
    /// </summary>
    @null,


    /// <summary>
    ///     布尔类型
    /// </summary>
    boolean,


    /// <summary>
    ///     数字类型
    /// </summary>
    number,


    /// <summary>
    ///     字符串类型
    /// </summary>
    @string,


    /// <summary>
    ///     数组类型
    /// </summary>
    array,


    /// <summary>
    ///     对象类型
    /// </summary>
    @object
}