namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     Block 信息
/// </summary>
public sealed class BlockInfo
{
    /// <summary>
    ///     Block 名称
    /// </summary>
    public string name { get; init; } = string.Empty;


    /// <summary>
    ///     Block 子节点
    /// </summary>
    public List<DejaVuTemplateNode> children { get; init; } = [];
}