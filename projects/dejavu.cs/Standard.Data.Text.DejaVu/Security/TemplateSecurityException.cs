namespace Std.Data.Text.DejaVu.Security;

/// <summary>
///     模板安全异常
/// </summary>
public class TemplateSecurityException : TemplateRenderException
{
    public TemplateSecurityException(string message) : base(message)
    {
    }
}