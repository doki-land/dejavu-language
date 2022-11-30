namespace Std.Data.Text.DejaVu.Loader;

/// <summary>
///     模板加载器接口
/// </summary>
public interface ITemplateLoader
{
    /// <summary>
    ///     加载模板
    /// </summary>
    Task<string> load(string path);


    /// <summary>
    ///     检查模板是否存在
    /// </summary>
    bool exists(string path);
}