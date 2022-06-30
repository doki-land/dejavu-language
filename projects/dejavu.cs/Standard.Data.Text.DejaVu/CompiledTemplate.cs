using Std.Data.Text.DejaVu.Optimizer;

namespace Std.Data.Text.DejaVu;

/// <summary>
///     编译后的模板产物——可缓存的优化 AST
/// </summary>
public sealed class CompiledTemplate
{
    /// <summary>
    ///     模板源路径
    /// </summary>
    public string template_path { get; init; } = string.Empty;


    /// <summary>
    ///     优化后的模板节点
    /// </summary>
    public List<DejaVuTemplateNode> nodes { get; init; } = [];


    /// <summary>
    ///     编译时间戳
    /// </summary>
    public DateTimeOffset compiled_at { get; init; }


    /// <summary>
    ///     源文件最后写入时间（用于缓存失效检测）
    /// </summary>
    public DateTimeOffset source_last_write_time { get; init; }


    /// <summary>
    ///     编译期符号表（变量作用域、block 名称、include 路径等）
    ///     ，按需生成，为 null 时表示未执行符号解析。
    /// </summary>
    public SymbolTable? symbol_table { get; init; }


    /// <summary>
    ///     编译后的渲染委托（AST → 表达式树 → JIT 编译）。
    ///     为 null 时表示尚未生成渲染委托。
    /// </summary>
    public Func<IDictionary<string, object>, string>? render_func { get; init; }


    /// <summary>
    ///     从解析结果编译模板
    /// </summary>
    public static CompiledTemplate compile(DejaVuParseResult parseResult, string templatePath,
        DateTimeOffset sourceLastWriteTime)
    {
        var optimizer = new TemplateOptimizer();
        var optimizedNodes = optimizer.optimize([.. parseResult.nodes]);

        return new CompiledTemplate
        {
            template_path = templatePath,
            nodes = optimizedNodes,
            compiled_at = DateTimeOffset.UtcNow,
            source_last_write_time = sourceLastWriteTime
        };
    }
}