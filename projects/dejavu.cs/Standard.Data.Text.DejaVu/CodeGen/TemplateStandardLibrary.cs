using Std.Data.Text.DejaVu.Optimizer;

namespace Std.Data.Text.DejaVu.CodeGen;

/// <summary>
///     模板标准库——跨语言一致的辅助函数定义。
///     每个辅助函数在 C#/TypeScript/Java 三种语言中行为一致。
/// </summary>
public sealed class TemplateStandardLibrary
{
    private readonly Dictionary<string, StandardHelper> _helpers = new();


    /// <summary>
    ///     创建模板标准库
    /// </summary>
    public TemplateStandardLibrary()
    {
        register_default_helpers();
    }


    /// <summary>
    ///     获取所有标准辅助函数
    /// </summary>
    public IReadOnlyDictionary<string, StandardHelper> helpers => _helpers;


    /// <summary>
    ///     获取标准辅助函数
    /// </summary>
    public StandardHelper? get_helper(string name)
    {
        return _helpers.GetValueOrDefault(name);
    }


    /// <summary>
    ///     注册标准辅助函数
    /// </summary>
    public void register(string name, StandardHelper helper)
    {
        _helpers[name] = helper;
    }


    /// <summary>
    ///     检查辅助函数是否已注册
    /// </summary>
    public bool has_helper(string name)
    {
        return _helpers.ContainsKey(name);
    }

    private void register_default_helpers()
    {
        register("formatDate", new StandardHelper(
            "formatDate",
            "格式化日期为指定格式",
            new HelperParameter("value", TemplateType.any, "日期值"),
            new HelperParameter("format", TemplateType.@string, "格式字符串，默认 yyyy-MM-dd"))
        {
            input_type = TemplateType.any,
            output_type = TemplateType.@string,
            ts_implementation = "(v, fmt) => v instanceof Date ? formatDate(v, fmt ?? 'yyyy-MM-dd') : String(v)",
            java_implementation =
                "(v, fmt) -> v instanceof java.time.temporal.Temporal ? v.toString() : String.valueOf(v)"
        });

        register("formatNumber", new StandardHelper(
            "formatNumber",
            "格式化数字为指定精度",
            new HelperParameter("value", TemplateType.number, "数字值"),
            new HelperParameter("decimals", TemplateType.number, "小数位数，默认 2"))
        {
            input_type = TemplateType.number,
            output_type = TemplateType.@string,
            ts_implementation = "(v, d) => Number(v).toFixed(d ?? 2)",
            java_implementation =
                "(v, d) -> String.format(\"%.\" + (d != null ? ((Number)d).intValue() : 2) + \"f\", ((Number)v).doubleValue())"
        });

        register("pluralize", new StandardHelper(
            "pluralize",
            "根据数量选择单数/复数形式",
            new HelperParameter("count", TemplateType.number, "数量"),
            new HelperParameter("singular", TemplateType.@string, "单数形式"),
            new HelperParameter("plural", TemplateType.@string, "复数形式"))
        {
            input_type = TemplateType.number,
            output_type = TemplateType.@string,
            ts_implementation = "(n, s, p) => n === 1 ? s : p",
            java_implementation = "(n, s, p) -> ((Number)n).intValue() == 1 ? s : p"
        });

        register("i18n", new StandardHelper(
            "i18n",
            "国际化翻译——根据键名查找翻译文本",
            new HelperParameter("key", TemplateType.@string, "翻译键名"))
        {
            input_type = TemplateType.@string,
            output_type = TemplateType.@string,
            ts_implementation = "(key) => __i18n[key] ?? key",
            java_implementation = "(key) -> __i18n.getOrDefault(key, key)"
        });

        register("truncateWords", new StandardHelper(
            "truncateWords",
            "按词数截断文本",
            new HelperParameter("text", TemplateType.@string, "文本"),
            new HelperParameter("wordCount", TemplateType.number, "保留词数，默认 30"))
        {
            input_type = TemplateType.@string,
            output_type = TemplateType.@string,
            ts_implementation = "(t, n) => t.split(' ').slice(0, n ?? 30).join(' ')",
            java_implementation =
                "(t, n) -> { int limit = n != null ? ((Number)n).intValue() : 30; var words = t.toString().split(\" \"); return Arrays.stream(words).limit(limit).collect(Collectors.joining(\" \")); }"
        });

        register("currency", new StandardHelper(
            "currency",
            "格式化为货币字符串",
            new HelperParameter("value", TemplateType.number, "金额"),
            new HelperParameter("symbol", TemplateType.@string, "货币符号，默认 ¥"))
        {
            input_type = TemplateType.number,
            output_type = TemplateType.@string,
            ts_implementation = "(v, sym) => (sym ?? '¥') + Number(v).toFixed(2)",
            java_implementation =
                "(v, sym) -> (sym != null ? sym : \"¥\") + String.format(\"%.2f\", ((Number)v).doubleValue())"
        });

        register("percentage", new StandardHelper(
            "percentage",
            "格式化为百分比字符串",
            new HelperParameter("value", TemplateType.number, "数值（0-1）"),
            new HelperParameter("decimals", TemplateType.number, "小数位数，默认 1"))
        {
            input_type = TemplateType.number,
            output_type = TemplateType.@string,
            ts_implementation = "(v, d) => (Number(v) * 100).toFixed(d ?? 1) + '%'",
            java_implementation =
                "(v, d) -> String.format(\"%.\" + (d != null ? ((Number)d).intValue() : 1) + \"f%%\", ((Number)v).doubleValue() * 100)"
        });

        register("stripTags", new StandardHelper(
            "stripTags",
            "移除 HTML 标签",
            new HelperParameter("html", TemplateType.@string, "HTML 文本"))
        {
            input_type = TemplateType.@string,
            output_type = TemplateType.@string,
            ts_implementation = "(html) => String(html).replace(/<[^>]*>/g, '')",
            java_implementation = "(html) -> html.toString().replaceAll(\"<[^>]*>\", \"\")"
        });

        register("urlEncode", new StandardHelper(
            "urlEncode",
            "URL 编码",
            new HelperParameter("value", TemplateType.@string, "待编码字符串"))
        {
            input_type = TemplateType.@string,
            output_type = TemplateType.@string,
            ts_implementation = "(v) => encodeURIComponent(String(v))",
            java_implementation =
                "(v) -> java.net.URLEncoder.encode(v.toString(), java.nio.charset.StandardCharsets.UTF_8)"
        });

        register("jsonEncode", new StandardHelper(
            "jsonEncode",
            "JSON 编码——将值序列化为 JSON 字符串",
            new HelperParameter("value", TemplateType.any, "待编码值"))
        {
            input_type = TemplateType.any,
            output_type = TemplateType.@string,
            ts_implementation = "(v) => JSON.stringify(v)",
            java_implementation = "(v) -> com.fasterxml.jackson.databind.ObjectMapper().writeValueAsString(v)"
        });

        register("defaultIfEmpty", new StandardHelper(
            "defaultIfEmpty",
            "值为空时返回默认值",
            new HelperParameter("value", TemplateType.any, "值"),
            new HelperParameter("defaultValue", TemplateType.any, "默认值"))
        {
            input_type = TemplateType.any,
            output_type = TemplateType.any,
            ts_implementation = "(v, d) => (v === null || v === undefined || v === '') ? d : v",
            java_implementation = "(v, d) -> (v == null || \"\".equals(v)) ? d : v"
        });

        register("sortBy", new StandardHelper(
            "sortBy",
            "按属性排序集合",
            new HelperParameter("collection", TemplateType.array, "集合"),
            new HelperParameter("property", TemplateType.@string, "排序属性名"))
        {
            input_type = TemplateType.array,
            output_type = TemplateType.array,
            ts_implementation = "(arr, prop) => [...arr].sort((a, b) => a[prop] < b[prop] ? -1 : 1)",
            java_implementation =
                "(arr, prop) -> ((List<?>) arr).stream().sorted(Comparator.comparing(o -> String.valueOf(((Map<?, ?>) o).get(prop)))).collect(Collectors.toList())"
        });
    }
}