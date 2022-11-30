namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     布局解析状态
/// </summary>
public enum LayoutResolveStatus
{
    /// <summary>
    ///     成功
    /// </summary>
    success,


    /// <summary>
    ///     循环继承
    /// </summary>
    circular_inheritance,


    /// <summary>
    ///     模板未找到
    /// </summary>
    template_not_found
}