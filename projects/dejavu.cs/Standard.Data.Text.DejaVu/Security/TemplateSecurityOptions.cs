using System.Net;

namespace Std.Data.Text.DejaVu.Security;

/// <summary>
///     模板安全设置
/// </summary>
public sealed class TemplateSecurityOptions
{
    /// <summary>
    ///     是否启用沙箱模式
    /// </summary>
    public bool enable_sandbox { get; init; } = true;


    /// <summary>
    ///     允许访问的类型列表
    /// </summary>
    public IReadOnlyList<Type> allowed_types { get; init; } = new List<Type>();


    /// <summary>
    ///     禁止访问的类型列表
    /// </summary>
    public IReadOnlyList<Type> blocked_types { get; init; } = new List<Type>
    {
        typeof(File),
        typeof(FileStream),
        typeof(Directory),
        typeof(WebClient),
        typeof(HttpClient),
        typeof(System.Diagnostics.Process)
    };


    /// <summary>
    ///     允许调用的方法列表
    /// </summary>
    public IReadOnlyList<string> allowed_methods { get; init; } = new List<string>();


    /// <summary>
    ///     禁止调用的方法列表
    /// </summary>
    public IReadOnlyList<string> blocked_methods { get; init; } = new List<string>
    {
        "GetType",
        "GetProperties",
        "GetFields",
        "GetMethods",
        "Invoke",
        "CreateInstance"
    };


    /// <summary>
    ///     最大循环迭代次数
    /// </summary>
    public int max_loop_iterations { get; init; } = 1000;


    /// <summary>
    ///     最大模板渲染时间（毫秒）
    /// </summary>
    public int max_render_time { get; init; } = 5000;
}