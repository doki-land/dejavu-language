namespace Std.Data.Text.DejaVu.Filters;

/// <summary>
///     委托过滤器
/// </summary>
public sealed class DelegateFilter : IFilter
{
    private readonly Func<object?, object?[], object?> _func;

    public DelegateFilter(Func<object?, object?> func)
    {
        _func = (value, _) => func(value);
    }

    public DelegateFilter(Func<object?, object?[], object?> func)
    {
        _func = func;
    }

    public object? apply(object? value, object?[] arguments)
    {
        return _func(value, arguments);
    }
}