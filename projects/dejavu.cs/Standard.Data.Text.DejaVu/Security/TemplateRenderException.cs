namespace Std.Data.Text.DejaVu.Security;

/// <summary>
///     模板渲染异常
/// </summary>
public class TemplateRenderException : Exception
{
    public TemplateRenderException(string message) : base(message)
    {
    }

    public TemplateRenderException(string message, Exception innerException) : base(message, innerException)
    {
    }
}