namespace Std.Data.Text.DejaVu.Security;

/// <summary>
///     HTML 输出上下文——决定转义策略
/// </summary>
public enum HtmlOutputContext
{
    /// <summary>
    ///     HTML 内容（默认）——转义 &lt; &gt; &amp; &quot; &#x27;
    /// </summary>
    html_content,


    /// <summary>
    ///     HTML 属性值——转义 &amp; &quot; &apos; &lt; &gt; + 控制字符
    /// </summary>
    html_attribute,


    /// <summary>
    ///     JavaScript 字符串——转义 \\ \' \" \n \r \t &lt; &gt; / + 控制字符
    /// </summary>
    java_script,


    /// <summary>
    ///     URL 参数——Uri.EscapeDataString
    /// </summary>
    url,


    /// <summary>
    ///     CSS 值——非字母数字字符转义
    /// </summary>
    css,


    /// <summary>
    ///     原始输出——不转义（用于 raw 块和已信任内容）
    /// </summary>
    raw
}