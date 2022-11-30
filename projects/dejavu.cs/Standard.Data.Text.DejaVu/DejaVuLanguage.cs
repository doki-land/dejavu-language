namespace Std.Data.Text.DejaVu;

/// <summary>
///     DejaVu 语言定义
/// </summary>
public sealed class DejaVuLanguage
{
    /// <summary>
    ///     Dora 模板语言（使用 &lt;% %&gt;）
    /// </summary>
    public static readonly DejaVuLanguage dora = new(
        "dora",
        "<%",
        "%>",
        "<%--",
        "--%>"
    );


    /// <summary>
    ///     Doki 模板语言（使用 {% %}）
    /// </summary>
    public static readonly DejaVuLanguage doki = new(
        "doki",
        "{%",
        "%}",
        "{%--",
        "--%}"
    );


    /// <summary>
    ///     创建 DejaVu 语言定义
    /// </summary>
    public DejaVuLanguage(string name, string openingDelimiter, string closingDelimiter, string commentStart,
        string commentEnd)
    {
        this.name = name;
        opening_delimiter = openingDelimiter;
        closing_delimiter = closingDelimiter;
        comment_start = commentStart;
        comment_end = commentEnd;
    }


    /// <summary>
    ///     代码块开始分隔符
    /// </summary>
    public string opening_delimiter { get; init; }


    /// <summary>
    ///     代码块结束分隔符
    /// </summary>
    public string closing_delimiter { get; init; }


    /// <summary>
    ///     注释开始分隔符
    /// </summary>
    public string comment_start { get; init; }


    /// <summary>
    ///     注释结束分隔符
    /// </summary>
    public string comment_end { get; init; }


    /// <summary>
    ///     语言名称
    /// </summary>
    public string name { get; init; }


    /// <summary>
    ///     根据名称获取语言定义
    /// </summary>
    public static DejaVuLanguage get_by_name(string name)
    {
        return name switch
        {
            "dora" => dora,
            "doki" => doki,
            _ => throw new ArgumentException($"Unknown language: {name}")
        };
    }
}