namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     主题定义
/// </summary>
public sealed class ThemeDefinition
{
    /// <summary>
    ///     主题名
    /// </summary>
    public string name { get; init; } = string.Empty;


    /// <summary>
    ///     基础主题名
    /// </summary>
    public string? base_theme_name { get; init; }


    /// <summary>
    ///     CSS 变量（已合并基础主题）
    /// </summary>
    public Dictionary<string, string> css_variables { get; init; } = new();


    /// <summary>
    ///     组件覆盖（组件名 → 覆盖模板源码）
    /// </summary>
    public Dictionary<string, string> component_overrides { get; init; } = new();


    /// <summary>
    ///     布局覆盖（布局名 → 覆盖模板源码）
    /// </summary>
    public Dictionary<string, string> layout_overrides { get; init; } = new();


    /// <summary>
    ///     继承深度
    /// </summary>
    public int inheritance_depth
    {
        get
        {
            var depth = 0;
            var current = base_theme_name;
            while (current != null)
            {
                depth++;
                current = null;
            }

            return depth;
        }
    }
}