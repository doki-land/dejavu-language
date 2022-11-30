namespace Std.Data.Text.DejaVu.Filters;

/// <summary>
///     过滤器接口
/// </summary>
public interface IFilter
{
    /// <summary>
    ///     应用过滤器
    /// </summary>
    object? apply(object? value, object?[] arguments);
}