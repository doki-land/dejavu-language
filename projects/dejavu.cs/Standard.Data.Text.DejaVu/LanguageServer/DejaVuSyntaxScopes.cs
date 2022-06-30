namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     DejaVu 语法高亮元数据——TextMate 兼容的 scope 分类。
///     用于 VSCode 扩展的 language configuration 和 TextMate 语法定义。
/// </summary>
public sealed class DejaVuSyntaxScopes
{
    /// <summary>
    ///     语言 ID
    /// </summary>
    public const string language_id = "dejavu";


    /// <summary>
    ///     语言显示名称
    /// </summary>
    public const string language_name = "DejaVu Template";

    /// <summary>
    ///     文件扩展名
    /// </summary>
    public static readonly string[] file_extensions = [".dejavu"];


    /// <summary>
    ///     获取所有 TextMate scope 定义
    /// </summary>
    public static List<ScopeDefinition> get_all_scopes()
    {
        return
        [
            new ScopeDefinition("comment.block.dejavu", "{%-- --%}", "模板注释"),
            new ScopeDefinition("keyword.control.dejavu", "if/else/loop/let/with/block/extends/include/raw/end",
                "控制关键字"),
            new ScopeDefinition("keyword.operator.pipe.dejavu", "|>", "管道运算符"),
            new ScopeDefinition("keyword.operator.colon.dejavu", ":", "参数分隔符"),
            new ScopeDefinition("variable.other.dejavu", "变量引用", "模板变量"),
            new ScopeDefinition("variable.other.loop-item.dejavu", "loop item 变量", "循环迭代变量"),
            new ScopeDefinition("variable.other.loop-index.dejavu", "index", "循环索引变量"),
            new ScopeDefinition("entity.name.function.filter.dejavu", "过滤器名称", "管道过滤器"),
            new ScopeDefinition("entity.name.function.helper.dejavu", "辅助函数名称", "标准库辅助函数"),
            new ScopeDefinition("entity.name.Type.block.dejavu", "block 名称", "块定义名称"),
            new ScopeDefinition("string.quoted.double.dejavu", "\"...\"", "双引号字符串"),
            new ScopeDefinition("string.quoted.single.dejavu", "'...'", "单引号字符串"),
            new ScopeDefinition("constant.numeric.dejavu", "数字字面量", "数字"),
            new ScopeDefinition("constant.language.boolean.dejavu", "true/false", "布尔值"),
            new ScopeDefinition("constant.language.null.dejavu", "null", "空值"),
            new ScopeDefinition("punctuation.definition.tag.begin.doki.dejavu", "{%", "Doki 标签开始"),
            new ScopeDefinition("punctuation.definition.tag.end.doki.dejavu", "%}", "Doki 标签结束"),
            new ScopeDefinition("punctuation.definition.tag.begin.dora.dejavu", "<%", "Dora 标签开始"),
            new ScopeDefinition("punctuation.definition.tag.end.dora.dejavu", "%>", "Dora 标签结束"),
            new ScopeDefinition("punctuation.definition.output.begin.dejavu", "{{", "输出标签开始"),
            new ScopeDefinition("punctuation.definition.output.end.dejavu", "}}", "输出标签结束"),
            new ScopeDefinition("meta.tag.template.dejavu", "{% ... %}", "模板标签"),
            new ScopeDefinition("meta.output.template.dejavu", "{{ ... }}", "输出标签"),
            new ScopeDefinition("support.function.filter.dejavu", "内置过滤器", "过滤器函数"),
            new ScopeDefinition("support.function.helper.dejavu", "标准库辅助函数", "Helper 函数")
        ];
    }


    /// <summary>
    ///     获取语言配置（括号匹配、注释切换等）
    /// </summary>
    public static LanguageConfiguration get_language_configuration()
    {
        return new LanguageConfiguration
        {
            comments = new CommentConfiguration
            {
                line_comment = null,
                block_comment = new CommentPair { open = "{%--", close = "--%}" }
            },
            brackets =
            [
                new BracketPair { open = "{", close = "}" },
                new BracketPair { open = "[", close = "]" },
                new BracketPair { open = "(", close = ")" },
                new BracketPair { open = "{%", close = "%}" },
                new BracketPair { open = "{{", close = "}}" }
            ],
            auto_closing_pairs =
            [
                new AutoClosingPair { open = "{", close = "}" },
                new AutoClosingPair { open = "[", close = "]" },
                new AutoClosingPair { open = "(", close = ")" },
                new AutoClosingPair { open = "\"", close = "\"" },
                new AutoClosingPair { open = "'", close = "'" }
            ],
            surrounding_pairs =
            [
                new SurroundingPair { open = "{", close = "}" },
                new SurroundingPair { open = "[", close = "]" },
                new SurroundingPair { open = "(", close = ")" },
                new SurroundingPair { open = "\"", close = "\"" },
                new SurroundingPair { open = "'", close = "'" }
            ],
            word_pattern = @"[a-zA-Z_]\w*",
            indentation_rules = new IndentationRules
            {
                increase_indent_pattern = @"\{%\s*(if|loop|let|with|block|raw|match)",
                decrease_indent_pattern = @"\{%\s*(end|else|else\s+if)"
            },
            folding = new FoldingConfiguration
            {
                markers = new FoldingMarkers
                {
                    start = @"\{%\s*(if|loop|let|with|block|raw|match)",
                    end = @"\{%\s*end\s*%\}"
                }
            }
        };
    }
}