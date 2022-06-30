namespace Std.Data.Text.DejaVu;

/// <summary>
///     include 节点
/// </summary>
public sealed class DejaVuIncludeNode : DejaVuTemplateNode
{
    /// <inheritdoc />
    public override DejaVuNodeType node_type => DejaVuNodeType.include;


    /// <summary>
    ///     包含的模板路径
    /// </summary>
    public string template_path { get; init; } = string.Empty;
}