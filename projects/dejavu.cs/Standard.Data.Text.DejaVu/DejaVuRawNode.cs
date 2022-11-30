namespace Std.Data.Text.DejaVu;

/// <summary>
///     raw 节点（原始 HTML 输出，不转义）
/// </summary>
public sealed class DejaVuRawNode : DejaVuTemplateNode
{
    /// <inheritdoc />
    public override DejaVuNodeType node_type => DejaVuNodeType.raw;


    /// <summary>
    ///     子节点
    /// </summary>
    public List<DejaVuTemplateNode> children { get; init; } = [];
}