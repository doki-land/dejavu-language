namespace Std.Data.Text.DejaVu.Loader;

/// <summary>
///     模板缓存
/// </summary>
public sealed class TemplateCache
{
    private readonly Dictionary<string, CacheEntry> _cache;
    private readonly TimeSpan _default_expiration;

    public TemplateCache(TimeSpan? defaultExpiration = null)
    {
        _cache = new Dictionary<string, CacheEntry>();
        _default_expiration = defaultExpiration ?? TimeSpan.FromMinutes(5);
    }


    /// <summary>
    ///     获取缓存的模板
    /// </summary>
    public string? get(string path)
    {
        if (_cache.TryGetValue(path, out var entry))
        {
            if (entry.expiration_time > DateTime.UtcNow) return entry.content;

            _cache.Remove(path);
        }

        return null;
    }


    /// <summary>
    ///     设置缓存
    /// </summary>
    public void set(string path, string content, TimeSpan? expiration = null)
    {
        _cache[path] = new CacheEntry
        {
            content = content,
            expiration_time = DateTime.UtcNow + (expiration ?? _default_expiration)
        };
    }


    /// <summary>
    ///     清除缓存
    /// </summary>
    public void clear()
    {
        _cache.Clear();
    }

    private class CacheEntry
    {
        public string content { get; init; } = string.Empty;
        public DateTime expiration_time { get; init; }
    }
}