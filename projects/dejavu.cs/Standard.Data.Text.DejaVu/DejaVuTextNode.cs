namespace Std.Data.Text.DejaVu;

/// <summary>
///     文本节点
/// </summary>
public sealed class DejaVuTextNode : DejaVuTemplateNode
{
    /// <inheritdoc />
    public override DejaVuNodeType node_type => DejaVuNodeType.text;


    /// <summary>
    ///     文本内容
    /// </summary>
    public string text { get; init; } = string.Empty;
}