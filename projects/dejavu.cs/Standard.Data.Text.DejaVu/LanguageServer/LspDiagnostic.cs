namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     LSP Diagnostic 对象
/// </summary>
public sealed class LspDiagnostic
{
    /// <summary>
    ///     诊断范围
    /// </summary>
    public LspRange range { get; init; } = new();


    /// <summary>
    ///     严重级别（1=Error, 2=Warning, 3=Info, 4=Hint）
    /// </summary>
    public int severity { get; init; }


    /// <summary>
    ///     诊断代码
    /// </summary>
    public string code { get; init; } = string.Empty;


    /// <summary>
    ///     诊断来源
    /// </summary>
    public string source { get; init; } = "dejavu";


    /// <summary>
    ///     诊断消息
    /// </summary>
    public string message { get; init; } = string.Empty;


    /// <summary>
    ///     相关信息
    /// </summary>
    public List<LspDiagnosticRelatedInformation>? related_information { get; init; }
}