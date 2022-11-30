namespace Std.Data.Text.DejaVu;

/// <summary>
///     block 节点
/// </summary>
public sealed class DejaVuBlockNode : DejaVuTemplateNode
{
    /// <inheritdoc />
    public override DejaVuNodeType node_type => DejaVuNodeType.block;


    /// <summary>
    ///     block 名称
    /// </summary>
    public string name { get; init; } = string.Empty;


    /// <summary>
    ///     子节点
    /// </summary>
    public List<DejaVuTemplateNode> children { get; init; } = [];
}