using System.Text;

namespace Std.Data.Text.DejaVu.Security;

/// <summary>
///     HTML 上下文感知转义器——根据输出上下文自动选择正确的转义策略。
/// </summary>
public sealed class HtmlEscaper
{
    /// <summary>
    ///     HTML 内容上下文转义——防 XSS 注入
    /// </summary>
    public static string escape_html_content(string? input)
    {
        if (string.IsNullOrEmpty(input)) return string.Empty;

        var sb = new StringBuilder(input.Length);

        foreach (var ch in input)
            switch (ch)
            {
                case '&':
                    sb.Append("&amp;");
                    break;
                case '<':
                    sb.Append("&lt;");
                    break;
                case '>':
                    sb.Append("&gt;");
                    break;
                case '"':
                    sb.Append("&quot;");
                    break;
                case '\'':
                    sb.Append("&#x27;");
                    break;
                default:
                    sb.Append(ch);
                    break;
            }

        return sb.ToString();
    }


    /// <summary>
    ///     HTML 属性上下文转义——属性值中的特殊字符
    /// </summary>
    public static string escape_html_attribute(string? input)
    {
        if (string.IsNullOrEmpty(input)) return string.Empty;

        var sb = new StringBuilder(input.Length);

        foreach (var ch in input)
            switch (ch)
            {
                case '&':
                    sb.Append("&amp;");
                    break;
                case '"':
                    sb.Append("&quot;");
                    break;
                case '\'':
                    sb.Append("&apos;");
                    break;
                case '<':
                    sb.Append("&lt;");
                    break;
                case '>':
                    sb.Append("&gt;");
                    break;
                default:
                    if (ch < 0x20)
                        sb.Append($"&#x{(int)ch:X};");
                    else
                        sb.Append(ch);

                    break;
            }

        return sb.ToString();
    }


    /// <summary>
    ///     JavaScript 上下文转义——防止脚本注入
    /// </summary>
    public static string escape_java_script(string? input)
    {
        if (string.IsNullOrEmpty(input)) return string.Empty;

        var sb = new StringBuilder(input.Length);

        foreach (var ch in input)
            switch (ch)
            {
                case '\\':
                    sb.Append("\\\\");
                    break;
                case '\'':
                    sb.Append("\\'");
                    break;
                case '"':
                    sb.Append("\\\"");
                    break;
                case '\n':
                    sb.Append("\\n");
                    break;
                case '\r':
                    sb.Append("\\r");
                    break;
                case '\t':
                    sb.Append("\\t");
                    break;
                case '<':
                    sb.Append("\\x3c");
                    break;
                case '>':
                    sb.Append("\\x3e");
                    break;
                case '/':
                    sb.Append("\\/");
                    break;
                default:
                    if (ch < 0x20)
                        sb.Append($"\\x{(int)ch:x2}");
                    else
                        sb.Append(ch);

                    break;
            }

        return sb.ToString();
    }


    /// <summary>
    ///     URL 上下文转义——防止 URL 注入
    /// </summary>
    public static string escape_url(string? input)
    {
        if (string.IsNullOrEmpty(input)) return string.Empty;

        return Uri.EscapeDataString(input);
    }


    /// <summary>
    ///     CSS 上下文转义——防止 CSS 注入
    /// </summary>
    public static string escape_css(string? input)
    {
        if (string.IsNullOrEmpty(input)) return string.Empty;

        var sb = new StringBuilder(input.Length);

        foreach (var ch in input)
            if (ch is >= 'a' and <= 'z' or >= 'A' and <= 'Z' or >= '0' and <= '9')
                sb.Append(ch);
            else
                sb.Append($"\\{ch}");

        return sb.ToString();
    }
}