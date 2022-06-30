namespace Std.Data.Text.DejaVu.Security;

/// <summary>
///     模板超时异常
/// </summary>
public class TemplateTimeoutException : TemplateRenderException
{
    public TemplateTimeoutException(string message) : base(message)
    {
    }
}