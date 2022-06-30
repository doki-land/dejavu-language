using System.Collections;
using System.Globalization;

namespace Std.Data.Text.DejaVu.Filters;

/// <summary>
///     过滤器注册表
/// </summary>
public sealed class FilterRegistry
{
    private readonly Dictionary<string, IFilter> _filters;

    public FilterRegistry()
    {
        _filters = new Dictionary<string, IFilter>();
        register_default_filters();
    }


    /// <summary>
    ///     注册过滤器
    /// </summary>
    public void register(string name, IFilter filter)
    {
        _filters[name] = filter;
    }


    /// <summary>
    ///     获取过滤器
    /// </summary>
    public IFilter? get(string name)
    {
        return _filters.GetValueOrDefault(name);
    }


    /// <summary>
    ///     检查过滤器是否已注册
    /// </summary>
    public bool has_filter(string name)
    {
        return _filters.ContainsKey(name);
    }


    /// <summary>
    ///     应用过滤器
    /// </summary>
    public object? apply(string name, object? value, object?[] arguments)
    {
        var filter = get(name);
        return filter?.apply(value, arguments);
    }


    /// <summary>
    ///     注册默认过滤器
    /// </summary>
    private void register_default_filters()
    {
        // 字符串过滤器
        register("uppercase", new DelegateFilter(value =>
            value?.ToString()?.ToUpperInvariant()));
        register("lowercase", new DelegateFilter(value =>
            value?.ToString()?.ToLowerInvariant()));
        register("trim", new DelegateFilter(value =>
            value?.ToString()?.Trim()));
        register("length", new DelegateFilter(value =>
            value?.ToString()?.Length ?? 0));
        register("reverse", new DelegateFilter(value =>
        {
            var str = value?.ToString();
            return str == null ? null : new string(str.Reverse().ToArray());
        }));

        // 数字过滤器
        register("abs", new DelegateFilter(value =>
            value is double d ? System.Math.Abs(d) : value));
        register("round", new DelegateFilter((value, args) =>
        {
            if (value is not double d) return value;

            var decimals = args.Length > 0 && args[0] is int i ? i : 0;
            return System.Math.Round(d, decimals);
        }));
        register("floor", new DelegateFilter(value =>
            value is double d ? System.Math.Floor(d) : value));
        register("ceil", new DelegateFilter(value =>
            value is double d ? System.Math.Ceiling(d) : value));

        // 集合过滤器
        register("first", new DelegateFilter(value =>
        {
            if (value is IEnumerable enumerable)
            {
                var enumerator = enumerable.GetEnumerator();
                return enumerator.MoveNext() ? enumerator.Current : null;
            }

            return null;
        }));
        register("last", new DelegateFilter(value =>
        {
            if (value is IEnumerable enumerable)
            {
                object? last = null;
                foreach (var item in enumerable) last = item;

                return last;
            }

            return null;
        }));
        register("count", new DelegateFilter(value =>
        {
            if (value is ICollection collection) return collection.Count;

            if (value is string str) return str.Length;

            return 0;
        }));
        register("join", new DelegateFilter((value, args) =>
        {
            if (value is not IEnumerable enumerable) return value;

            var separator = args.Length > 0 ? args[0]?.ToString() ?? ", " : ", ";
            var items = new List<string>();
            foreach (var item in enumerable) items.Add(item?.ToString() ?? "");

            return string.Join(separator, items);
        }));

        // 日期过滤器
        register("date", new DelegateFilter((value, args) =>
        {
            if (value is not DateTime dt) return value;

            var format = args.Length > 0 ? args[0]?.ToString() ?? "yyyy-MM-dd" : "yyyy-MM-dd";
            return dt.ToString(format, CultureInfo.InvariantCulture);
        }));
        register("datetime", new DelegateFilter((value, args) =>
        {
            if (value is not DateTime dt) return value;

            var format = args.Length > 0 ? args[0]?.ToString() ?? "yyyy-MM-dd HH:mm:ss" : "yyyy-MM-dd HH:mm:ss";
            return dt.ToString(format, CultureInfo.InvariantCulture);
        }));

        // 默认值过滤器
        register("default", new DelegateFilter((value, args) =>
        {
            if (value != null && !string.IsNullOrEmpty(value.ToString())) return value;

            return args.Length > 0 ? args[0] : null;
        }));

        // HTML 过滤器
        register("escape", new DelegateFilter(value =>
        {
            var str = value?.ToString();
            if (str == null) return null;

            return str
                .Replace("&", "&amp;")
                .Replace("<", "&lt;")
                .Replace(">", "&gt;")
                .Replace("\"", "&quot;")
                .Replace("'", "&#x27;");
        }));
        register("safe", new DelegateFilter(value => value));
    }
}