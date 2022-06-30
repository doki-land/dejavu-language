namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     LSP DiagnosticRelatedInformation 对象
/// </summary>
public sealed class LspDiagnosticRelatedInformation
{
    /// <summary>
    ///     相关位置
    /// </summary>
    public LspLocation location { get; init; } = new();


    /// <summary>
    ///     相关消息
    /// </summary>
    public string message { get; init; } = string.Empty;
}