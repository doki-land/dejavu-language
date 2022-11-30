namespace Std.Data.Text.DejaVu.Loader;

/// <summary>
///     模板管理器
/// </summary>
public sealed class TemplateManager
{
    private readonly TemplateCache _cache;
    private readonly ITemplateLoader _loader;

    public TemplateManager(ITemplateLoader loader)
    {
        _loader = loader;
        _cache = new TemplateCache();
    }


    /// <summary>
    ///     加载模板
    /// </summary>
    public async Task<string> load(string path)
    {
        // 检查缓存
        var cached = _cache.get(path);
        if (cached != null) return cached;

        // 加载模板
        var content = await _loader.load(path);

        // 缓存结果
        _cache.set(path, content);

        return content;
    }


    /// <summary>
    ///     清除缓存
    /// </summary>
    public void clear_cache()
    {
        _cache.clear();
    }
}