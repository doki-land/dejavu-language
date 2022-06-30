namespace Std.Data.Text.DejaVu;

/// <summary>
///     extends 节点
/// </summary>
public sealed class DejaVuExtendsNode : DejaVuTemplateNode
{
    /// <inheritdoc />
    public override DejaVuNodeType node_type => DejaVuNodeType.extends;


    /// <summary>
    ///     父模板路径
    /// </summary>
    public string parent_template { get; init; } = string.Empty;
}