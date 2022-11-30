namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     Scope 定义
/// </summary>
public sealed class ScopeDefinition
{
    /// <summary>
    ///     创建 Scope 定义
    /// </summary>
    public ScopeDefinition(string scope, string pattern, string description)
    {
        this.scope = scope;
        this.pattern = pattern;
        this.description = description;
    }

    /// <summary>
    ///     TextMate scope 名称
    /// </summary>
    public string scope { get; }


    /// <summary>
    ///     匹配模式描述
    /// </summary>
    public string pattern { get; }


    /// <summary>
    ///     中文描述
    /// </summary>
    public string description { get; }
}