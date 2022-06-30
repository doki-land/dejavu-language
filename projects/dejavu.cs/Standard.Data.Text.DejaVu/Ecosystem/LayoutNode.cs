namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     布局节点
/// </summary>
public sealed class LayoutNode
{
    /// <summary>
    ///     模板路径
    /// </summary>
    public string template_path { get; init; } = string.Empty;


    /// <summary>
    ///     模板源码
    /// </summary>
    public string source { get; init; } = string.Empty;


    /// <summary>
    ///     定义的 block
    /// </summary>
    public Dictionary<string, BlockInfo> blocks { get; init; } = new();


    /// <summary>
    ///     Content Placeholder 列表
    /// </summary>
    public List<ContentPlaceholder> content_placeholders { get; init; } = [];


    /// <summary>
    ///     父模板路径
    /// </summary>
    public string? parent_template_path { get; init; }
}