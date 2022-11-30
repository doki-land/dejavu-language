namespace Std.Data.Text.DejaVu.Loader;

/// <summary>
///     文件系统模板加载器
/// </summary>
public sealed class FileSystemTemplateLoader : ITemplateLoader
{
    private readonly string _base_path;

    public FileSystemTemplateLoader(string basePath)
    {
        _base_path = basePath;
    }


    /// <inheritdoc />
    public async Task<string> load(string path)
    {
        var fullPath = Path.Combine(_base_path, path);
        if (!File.Exists(fullPath)) throw new FileNotFoundException($"Template not found: {path}", fullPath);

        return await File.ReadAllTextAsync(fullPath);
    }


    /// <inheritdoc />
    public bool exists(string path)
    {
        var fullPath = Path.Combine(_base_path, path);
        return File.Exists(fullPath);
    }
}