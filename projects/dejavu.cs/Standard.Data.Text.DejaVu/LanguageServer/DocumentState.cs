using Std.Data.Text.DejaVu.Optimizer;

namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     文档状态缓存
/// </summary>
public sealed class DocumentState
{
    /// <summary>
    ///     文档源码
    /// </summary>
    public string source { get; init; } = string.Empty;


    /// <summary>
    ///     优化后的 AST 节点
    /// </summary>
    public List<DejaVuTemplateNode> nodes { get; init; } = [];


    /// <summary>
    ///     符号表
    /// </summary>
    public SymbolTable? symbol_table { get; init; }


    /// <summary>
    ///     推导出的变量类型
    /// </summary>
    public Dictionary<string, TemplateType> inferred_types { get; init; } = new();


    /// <summary>
    ///     LSP 诊断列表
    /// </summary>
    public List<LspDiagnostic> diagnostics { get; init; } = [];
}