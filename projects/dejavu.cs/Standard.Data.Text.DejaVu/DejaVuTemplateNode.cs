namespace Std.Data.Text.DejaVu;

/// <summary>
///     DejaVu 模板节点基类
/// </summary>
public abstract class DejaVuTemplateNode
{
    /// <summary>
    ///     节点类型
    /// </summary>
    public abstract DejaVuNodeType node_type { get; }


    /// <summary>
    ///     源码行号（1-based，0 表示未知）
    /// </summary>
    public int source_line { get; init; }


    /// <summary>
    ///     源码列号（1-based，0 表示未知）
    /// </summary>
    public int source_column { get; init; }
}