namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     模板加载器接口
/// </summary>
public interface ITemplateLoader
{
    /// <summary>
    ///     加载模板源码
    /// </summary>
    string? load(string templatePath);


    /// <summary>
    ///     解析模板路径
    /// </summary>
    string resolve_path(string templateName);
}