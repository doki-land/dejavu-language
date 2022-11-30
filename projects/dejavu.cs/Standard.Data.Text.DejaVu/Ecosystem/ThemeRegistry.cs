using System.Text;

namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     主题注册表——模板主题包（CSS 变量 + 组件覆盖 + 布局覆盖）。
///     支持主题继承和变量覆盖。
/// </summary>
public sealed class ThemeRegistry
{
    private readonly Dictionary<string, ThemeDefinition> _themes = new();


    /// <summary>
    ///     所有已注册主题
    /// </summary>
    public IReadOnlyDictionary<string, ThemeDefinition> themes => _themes;


    /// <summary>
    ///     当前激活的主题名
    /// </summary>
    public string? active_theme_name { get; private set; }


    /// <summary>
    ///     获取当前激活的主题
    /// </summary>
    public ThemeDefinition? active_theme =>
        active_theme_name != null && _themes.TryGetValue(active_theme_name, out var theme) ? theme : null;


    /// <summary>
    ///     注册主题
    /// </summary>
    /// <param name="name">主题名。</param>
    /// <param name="baseThemeName">基础主题名（主题继承）。</param>
    /// <param name="cssVariables">CSS 变量覆盖。</param>
    /// <param name="componentOverrides">组件覆盖。</param>
    /// <param name="layoutOverrides">布局覆盖。</param>
    public void register(string name, string? baseThemeName = null, Dictionary<string, string>? cssVariables = null,
        Dictionary<string, string>? componentOverrides = null, Dictionary<string, string>? layoutOverrides = null)
    {
        var resolvedVariables = new Dictionary<string, string>();
        var resolvedComponents = new Dictionary<string, string>();
        var resolvedLayouts = new Dictionary<string, string>();

        if (!string.IsNullOrEmpty(baseThemeName) && _themes.TryGetValue(baseThemeName, out var baseTheme))
        {
            foreach (var (key, value) in baseTheme.css_variables) resolvedVariables[key] = value;

            foreach (var (key, value) in baseTheme.component_overrides) resolvedComponents[key] = value;

            foreach (var (key, value) in baseTheme.layout_overrides) resolvedLayouts[key] = value;
        }

        if (cssVariables != null)
            foreach (var (key, value) in cssVariables)
                resolvedVariables[key] = value;

        if (componentOverrides != null)
            foreach (var (key, value) in componentOverrides)
                resolvedComponents[key] = value;

        if (layoutOverrides != null)
            foreach (var (key, value) in layoutOverrides)
                resolvedLayouts[key] = value;

        _themes[name] = new ThemeDefinition
        {
            name = name,
            base_theme_name = baseThemeName,
            css_variables = resolvedVariables,
            component_overrides = resolvedComponents,
            layout_overrides = resolvedLayouts
        };
    }


    /// <summary>
    ///     激活主题
    /// </summary>
    public void activate(string name)
    {
        if (!_themes.ContainsKey(name)) throw new InvalidOperationException($"主题 \"{name}\" 未注册");

        active_theme_name = name;
    }


    /// <summary>
    ///     获取主题
    /// </summary>
    public ThemeDefinition? get_theme(string name)
    {
        return _themes.GetValueOrDefault(name);
    }


    /// <summary>
    ///     生成当前主题的 CSS 变量声明
    /// </summary>
    /// <param name="selector">CSS 选择器，默认 :root。</param>
    /// <returns>CSS 源码。</returns>
    public string generate_css_variables(string selector = ":root")
    {
        var theme = active_theme;
        if (theme == null || theme.css_variables.Count == 0) return string.Empty;

        var sb = new StringBuilder();
        sb.AppendLine($"{selector} {{");

        foreach (var (key, value) in theme.css_variables.OrderBy(kv => kv.Key)) sb.AppendLine($"    --{key}: {value};");

        sb.AppendLine("}");
        return sb.ToString();
    }


    /// <summary>
    ///     获取组件覆盖模板
    /// </summary>
    /// <param name="componentName">组件名。</param>
    /// <returns>覆盖模板源码，或 null。</returns>
    public string? get_component_override(string componentName)
    {
        return active_theme?.component_overrides.TryGetValue(componentName, out var source) == true
            ? source
            : null;
    }


    /// <summary>
    ///     获取布局覆盖模板
    /// </summary>
    /// <param name="layoutName">布局名。</param>
    /// <returns>覆盖模板源码，或 null。</returns>
    public string? get_layout_override(string layoutName)
    {
        return active_theme?.layout_overrides.TryGetValue(layoutName, out var source) == true
            ? source
            : null;
    }


    /// <summary>
    ///     解析 CSS 变量引用——将 var(--xxx) 替换为实际值
    /// </summary>
    /// <param name="input">包含 var() 引用的字符串。</param>
    /// <returns>替换后的字符串。</returns>
    public string resolve_css_variables(string input)
    {
        var theme = active_theme;
        if (theme == null) return input;

        foreach (var (key, value) in theme.css_variables) input = input.Replace($"var(--{key})", value);

        return input;
    }
}