namespace Std.Data.Text.DejaVu;

/// <summary>
///     super 节点（渲染父模板的 block 默认内容）
/// </summary>
public sealed class DejaVuSuperNode : DejaVuTemplateNode
{
    /// <inheritdoc />
    public override DejaVuNodeType node_type => DejaVuNodeType.super;
}