namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     表达式令牌
/// </summary>
public sealed class ExpressionToken
{
    /// <summary>
    ///     创建表达式令牌
    /// </summary>
    /// <param name="type">令牌类型。</param>
    /// <param name="value">令牌值。</param>
    public ExpressionToken(ExpressionTokenType type, object? value)
    {
        this.type = type;
        this.value = value;
    }


    /// <summary>
    ///     令牌类型
    /// </summary>
    public ExpressionTokenType type { get; }


    /// <summary>
    ///     值
    /// </summary>
    public object? value { get; }
}