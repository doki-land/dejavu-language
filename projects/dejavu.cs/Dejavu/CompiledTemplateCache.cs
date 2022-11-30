namespace Std.Template.Dejavu;

/// <summary>
///     编译模板缓存——基于源文件最后写入时间的缓存失效检测。
/// </summary>
public sealed class CompiledTemplateCache
{
    private readonly Dictionary<string, CacheEntry> _cache = new();
    private readonly object _lock = new();

    /// <summary>
    ///     当前缓存的模板数量
    /// </summary>
    public int Count
    {
        get
        {
            lock (_lock)
            {
                return _cache.Count;
            }
        }
    }

    /// <summary>
    ///     获取或编译模板
    /// </summary>
    /// <param name="templatePath">模板源文件路径</param>
    /// <param name="source">模板源内容</param>
    /// <param name="parser">解析器</param>
    /// <returns>编译后的模板</returns>
    public CompiledTemplate GetOrCompile(string templatePath, string source, DejaVuParser parser)
    {
        var lastWriteTime = GetSourceLastWriteTime(templatePath);

        lock (_lock)
        {
            if (_cache.TryGetValue(templatePath, out var entry) && entry.SourceLastWriteTime == lastWriteTime)
                return entry.Template;

            var compiled = parser.Compile(source, templatePath);
            _cache[templatePath] = new CacheEntry(compiled, lastWriteTime);
            return compiled;
        }
    }

    /// <summary>
    ///     预热缓存——预编译模板并存入缓存
    /// </summary>
    public void Warmup(string templatePath, string source, DejaVuParser parser)
    {
        GetOrCompile(templatePath, source, parser);
    }

    /// <summary>
    ///     使指定模板的缓存失效
    /// </summary>
    public void Invalidate(string templatePath)
    {
        lock (_lock)
        {
            _cache.Remove(templatePath);
        }
    }

    /// <summary>
    ///     清空所有缓存
    /// </summary>
    public void Clear()
    {
        lock (_lock)
        {
            _cache.Clear();
        }
    }

    private static DateTimeOffset GetSourceLastWriteTime(string templatePath)
    {
        if (string.IsNullOrEmpty(templatePath) || !File.Exists(templatePath)) return DateTimeOffset.MinValue;

        return File.GetLastWriteTimeUtc(templatePath);
    }

    private sealed record CacheEntry(CompiledTemplate Template, DateTimeOffset SourceLastWriteTime);
}