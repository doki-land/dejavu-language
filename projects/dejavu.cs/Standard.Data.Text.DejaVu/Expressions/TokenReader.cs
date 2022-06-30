namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     令牌读取器
/// </summary>
public sealed class TokenReader
{
    private readonly List<ExpressionToken> _tokens;
    private int _position;


    /// <summary>
    ///     创建令牌读取器
    /// </summary>
    /// <param name="tokens">令牌列表。</param>
    public TokenReader(List<ExpressionToken> tokens)
    {
        _tokens = tokens;
        _position = 0;
    }


    /// <summary>
    ///     是否已结束
    /// </summary>
    public bool is_at_end => _position >= _tokens.Count;


    /// <summary>
    ///     当前令牌
    /// </summary>
    public ExpressionToken current => _tokens[_position];


    /// <summary>
    ///     前进
    /// </summary>
    public ExpressionToken advance()
    {
        return _tokens[_position++];
    }
}