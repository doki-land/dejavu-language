namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     LSP CompletionItem 对象
/// </summary>
public sealed class LspCompletionItem
{
    /// <summary>
    ///     补全标签
    /// </summary>
    public string label { get; init; } = string.Empty;


    /// <summary>
    ///     补全类型（1=Text, 2=Method, 3=Function, 5=Field, 6=Variable, 9=Module, 11=File, 12=Folder, 13=Class, 14=Interface,
    ///     15=Color, 17=Keyword, 18=Snippet）
    /// </summary>
    public int kind { get; init; }


    /// <summary>
    ///     补全详情
    /// </summary>
    public string detail { get; init; } = string.Empty;


    /// <summary>
    ///     补全文档
    /// </summary>
    public string documentation { get; init; } = string.Empty;


    /// <summary>
    ///     插入文本
    /// </summary>
    public string insert_text { get; init; } = string.Empty;
}