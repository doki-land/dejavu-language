namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     语言配置
/// </summary>
public sealed class LanguageConfiguration
{
    /// <summary>
    ///     注释配置
    /// </summary>
    public CommentConfiguration comments { get; init; } = new();


    /// <summary>
    ///     括号对
    /// </summary>
    public List<BracketPair> brackets { get; init; } = [];


    /// <summary>
    ///     自动关闭对
    /// </summary>
    public List<AutoClosingPair> auto_closing_pairs { get; init; } = [];


    /// <summary>
    ///     环绕对
    /// </summary>
    public List<SurroundingPair> surrounding_pairs { get; init; } = [];


    /// <summary>
    ///     单词模式
    /// </summary>
    public string word_pattern { get; init; } = string.Empty;


    /// <summary>
    ///     缩进规则
    /// </summary>
    public IndentationRules indentation_rules { get; init; } = new();


    /// <summary>
    ///     折叠配置
    /// </summary>
    public FoldingConfiguration folding { get; init; } = new();
}