namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     表达式令牌类型
/// </summary>
public enum ExpressionTokenType
{
    number,
    @string,
    boolean,
    identifier,
    plus,
    minus,
    multiply,
    divide,
    modulo,
    equal,
    not_equal,
    less_than,
    less_than_or_equal,
    greater_than,
    greater_than_or_equal,
    and,
    or,
    not,
    pipe,
    left_paren,
    right_paren,
    left_bracket,
    right_bracket,
    comma,
    dot,
    colon
}