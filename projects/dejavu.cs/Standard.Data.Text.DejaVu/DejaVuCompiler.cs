using Std.Data.Text.DejaVu.CodeGen;
using Std.Data.Text.DejaVu.Filters;
using Std.Data.Text.DejaVu.Optimizer;
using Std.Data.Text.Diagnostics;

namespace Std.Data.Text.DejaVu;

/// <summary>
///     DejaVu 模板编译器——协调解析、优化、符号解析、错误收集的编译管线。
/// </summary>
public sealed class DejaVuCompiler
{
    private readonly TemplateOptimizer _optimizer;
    private readonly DejaVuParser _parser;
    private readonly SymbolResolver _symbol_resolver;


    /// <summary>
    ///     创建模板编译器
    /// </summary>
    /// <param name="parser">模板解析器。</param>
    public DejaVuCompiler(DejaVuParser parser)
    {
        _parser = parser;
        _optimizer = new TemplateOptimizer();
        _symbol_resolver = new SymbolResolver(new DiagnosticSink());
    }


    /// <summary>
    ///     编译模板源码——解析 + 优化 + 符号解析，返回可缓存的编译产物
    /// </summary>
    /// <param name="source">模板源码。</param>
    /// <param name="templatePath">模板源文件路径（可选，用于缓存失效和错误定位）。</param>
    /// <param name="emitSymbolTable">是否输出符号表（用于 IDE 智能提示等）。</param>
    /// <param name="emitRenderFunc">是否生成渲染委托（JIT 编译，用于高性能渲染）。</param>
    /// <returns>编译后的模板。</returns>
    public CompiledTemplate compile(string source, string templatePath = "", bool emitSymbolTable = false,
        bool emitRenderFunc = false)
    {
        var parseResult = _parser.parse(source);
        var optimizedNodes = _optimizer.optimize([.. parseResult.nodes]);

        var symbolTable = _symbol_resolver.resolve(optimizedNodes);

        Func<IDictionary<string, object>, string>? renderFunc = null;
        if (emitRenderFunc)
        {
            var codeGen = new TemplateCodeGenerator();
            renderFunc = codeGen.compile(optimizedNodes);
        }

        return new CompiledTemplate
        {
            template_path = templatePath,
            nodes = optimizedNodes,
            compiled_at = DateTimeOffset.UtcNow,
            source_last_write_time = get_source_last_write_time(templatePath),
            symbol_table = emitSymbolTable ? symbolTable : null,
            render_func = renderFunc
        };
    }


    /// <summary>
    ///     生成 TypeScript 渲染函数源码
    /// </summary>
    /// <param name="source">模板源码。</param>
    /// <param name="templateName">模板函数名。</param>
    /// <param name="options">TypeScript 生成选项。</param>
    /// <returns>TypeScript 源码。</returns>
    public string compile_to_type_script(string source, string templateName = "render",
        TypeScriptGeneratorOptions? options = null)
    {
        var parseResult = _parser.parse(source);
        var optimizedNodes = _optimizer.optimize([.. parseResult.nodes]);
        var generator = options != null
            ? new TypeScriptCodeGenerator(options)
            : new TypeScriptCodeGenerator();
        return generator.generate(optimizedNodes, templateName);
    }


    /// <summary>
    ///     从模板推导 TypeScript Data 接口
    /// </summary>
    /// <param name="source">模板源码。</param>
    /// <param name="interfaceName">接口名称。</param>
    /// <returns>TypeScript 接口源码。</returns>
    public string infer_type_script_interface(string source, string interfaceName = "TemplateData")
    {
        var parseResult = _parser.parse(source);
        var optimizedNodes = _optimizer.optimize([.. parseResult.nodes]);
        var inferrer = new TypeScriptTypeInferrer();
        return inferrer.infer_interface(optimizedNodes, interfaceName);
    }


    /// <summary>
    ///     执行编译期类型检查
    /// </summary>
    /// <param name="source">模板源码。</param>
    /// <param name="knownTypes">已知变量类型（从外部 Data 类型注解提供）。</param>
    /// <returns>推导出的变量类型表。</returns>
    public Dictionary<string, TemplateType> check_types(string source,
        Dictionary<string, TemplateType>? knownTypes = null)
    {
        var parseResult = _parser.parse(source);
        var optimizedNodes = _optimizer.optimize([.. parseResult.nodes]);
        var typeChecker = new TypeChecker(new DiagnosticSink(), new FilterRegistry());
        return typeChecker.check(optimizedNodes, knownTypes);
    }


    /// <summary>
    ///     生成 Java 渲染类源码
    /// </summary>
    /// <param name="source">模板源码。</param>
    /// <param name="className">Java 类名。</param>
    /// <param name="options">Java 生成选项。</param>
    /// <returns>Java 源码。</returns>
    public string compile_to_java(string source, string className = "TemplateRenderer",
        JavaGeneratorOptions? options = null)
    {
        var parseResult = _parser.parse(source);
        var optimizedNodes = _optimizer.optimize([.. parseResult.nodes]);
        var generator = options != null
            ? new JavaCodeGenerator(options)
            : new JavaCodeGenerator();
        return generator.generate(optimizedNodes, className);
    }

    private static DateTimeOffset get_source_last_write_time(string templatePath)
    {
        if (string.IsNullOrEmpty(templatePath) || !File.Exists(templatePath)) return DateTimeOffset.MinValue;

        return File.GetLastWriteTimeUtc(templatePath);
    }
}