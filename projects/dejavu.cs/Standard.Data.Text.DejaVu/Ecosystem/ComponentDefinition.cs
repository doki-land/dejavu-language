using System.Text;

namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     组件定义
/// </summary>
public sealed class ComponentDefinition
{
    /// <summary>
    ///     组件名
    /// </summary>
    public string name { get; init; } = string.Empty;


    /// <summary>
    ///     组件模板源码
    /// </summary>
    public string template_source { get; init; } = string.Empty;


    /// <summary>
    ///     解析后的节点列表
    /// </summary>
    public List<DejaVuTemplateNode> nodes { get; init; } = [];


    /// <summary>
    ///     组件参数定义
    /// </summary>
    public List<ComponentProp> props { get; init; } = [];


    /// <summary>
    ///     组件插槽定义
    /// </summary>
    public List<ComponentSlot> slots { get; init; } = [];


    /// <summary>
    ///     生成组件调用签名
    /// </summary>
    public string signature
    {
        get
        {
            var sb = new StringBuilder();
            sb.Append(name);
            sb.Append('(');
            sb.Append(string.Join(", ", props.Select(p => $"{p.name}: {p.type}")));
            sb.Append(')');

            if (slots.Count > 0)
            {
                sb.Append(" slots: [");
                sb.Append(string.Join(", ", slots.Select(s => s.name)));
                sb.Append(']');
            }

            return sb.ToString();
        }
    }
}