namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     组件插槽定义
/// </summary>
public sealed class ComponentSlot
{
    /// <summary>
    ///     插槽名
    /// </summary>
    public string name { get; init; } = string.Empty;


    /// <summary>
    ///     插槽描述
    /// </summary>
    public string description { get; init; } = string.Empty;


    /// <summary>
    ///     是否有默认内容
    /// </summary>
    public bool has_default_content { get; init; }
}