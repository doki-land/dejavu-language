namespace Std.Data.Text.DejaVu.Loader;

/// <summary>
///     内存模板加载器
/// </summary>
public sealed class MemoryTemplateLoader : ITemplateLoader
{
    private readonly Dictionary<string, string> _templates;

    public MemoryTemplateLoader()
    {
        _templates = new Dictionary<string, string>();
    }


    /// <inheritdoc />
    public Task<string> load(string path)
    {
        if (!_templates.TryGetValue(path, out var content))
            throw new KeyNotFoundException($"Template not found: {path}");

        return Task.FromResult(content);
    }


    /// <inheritdoc />
    public bool exists(string path)
    {
        return _templates.ContainsKey(path);
    }


    /// <summary>
    ///     添加模板
    /// </summary>
    public void add(string path, string content)
    {
        _templates[path] = content;
    }
}