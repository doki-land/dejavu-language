using Std.Data.Text.Diagnostics;
using Std.Data.Text.Syntax;

namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     LSP 诊断转换器——将 Oak Diagnostic 转换为 LSP 兼容的 JSON 结构。
///     输出可直接序列化为 LSP `textDocument/publishDiagnostics` 的 `Diagnostic[]`。
/// </summary>
public sealed class LspDiagnosticConverter
{
    /// <summary>
    ///     将 Oak 诊断消息转换为 LSP 诊断对象列表
    /// </summary>
    /// <param name="diagnostics">Oak 诊断消息列表。</param>
    /// <returns>LSP 诊断对象列表。</returns>
    public static List<LspDiagnostic> convert(IReadOnlyList<Diagnostic> diagnostics)
    {
        var result = new List<LspDiagnostic>(diagnostics.Count);

        foreach (var diag in diagnostics) result.Add(ConvertOne(diag));

        return result;
    }


    /// <summary>
    ///     将 Oak DiagnosticSink 转换为 LSP 诊断对象列表
    /// </summary>
    public static List<LspDiagnostic> convert(DiagnosticSink sink)
    {
        return convert(sink.messages);
    }

    private static LspDiagnostic ConvertOne(Diagnostic diag)
    {
        return new LspDiagnostic
        {
            range = text_span_to_lsp_range(diag.span),
            severity = severity_to_lsp(diag.severity),
            code = string.Empty,
            source = "dejavu",
            message = diag.message
        };
    }

    private static LspRange text_span_to_lsp_range(TextSpan span)
    {
        if (span == default)
            return new LspRange
            {
                start = new LspPosition { line = 0, character = 0 }, end = new LspPosition { line = 0, character = 0 }
            };

        return new LspRange
        {
            start = new LspPosition { line = 0, character = span.start },
            end = new LspPosition { line = 0, character = span.end }
        };
    }

    private static int severity_to_lsp(DiagnosticSeverity severity)
    {
        return severity switch
        {
            DiagnosticSeverity.fatal => 1,
            DiagnosticSeverity.error => 1,
            DiagnosticSeverity.warning => 2,
            DiagnosticSeverity.info => 3,
            DiagnosticSeverity.hint => 4,
            DiagnosticSeverity.debug => 3,
            DiagnosticSeverity.trace => 4,
            _ => 3
        };
    }
}
