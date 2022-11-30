namespace Std.Data.Text.DejaVu;

/// <summary>
///     DejaVu 节点类型
/// </summary>
public enum DejaVuNodeType
{
    /// <summary>
    ///     文本节点
    /// </summary>
    text,


    /// <summary>
    ///     代码节点
    /// </summary>
    code,


    /// <summary>
    ///     if 节点
    /// </summary>
    @if,


    /// <summary>
    ///     loop 节点
    /// </summary>
    loop,


    /// <summary>
    ///     match 节点
    /// </summary>
    match,


    /// <summary>
    ///     block 节点
    /// </summary>
    block,


    /// <summary>
    ///     extends 节点
    /// </summary>
    extends,


    /// <summary>
    ///     include 节点
    /// </summary>
    include,


    /// <summary>
    ///     let 节点
    /// </summary>
    let,


    /// <summary>
    ///     with 节点
    /// </summary>
    with,


    /// <summary>
    ///     super 节点
    /// </summary>
    super,


    /// <summary>
    ///     raw 节点（原始输出）
    /// </summary>
    raw
}