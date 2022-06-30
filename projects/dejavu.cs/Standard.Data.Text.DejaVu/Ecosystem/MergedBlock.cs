namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     合并后的 Block
/// </summary>
public sealed class MergedBlock
{
    /// <summary>
    ///     Block 名称
    /// </summary>
    public string name { get; init; } = string.Empty;


    /// <summary>
    ///     默认内容（来自最初定义）
    /// </summary>
    public List<DejaVuTemplateNode> default_content { get; init; } = [];


    /// <summary>
    ///     覆盖内容（来自子模板）
    /// </summary>
    public List<DejaVuTemplateNode> override_content { get; init; } = [];


    /// <summary>
    ///     定义位置
    /// </summary>
    public string defined_in { get; init; } = string.Empty;


    /// <summary>
    ///     覆盖来源
    /// </summary>
    public string override_from { get; init; } = string.Empty;
}