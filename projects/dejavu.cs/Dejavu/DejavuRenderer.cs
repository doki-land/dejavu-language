using System.Collections;
using System.Text;

namespace Std.Template.Dejavu;

/// <summary>
///     渲染上下文
/// </summary>
public class RenderContext
{
    /// <summary>
    ///     模板变量表
    /// </summary>
    public Dictionary<string, object> Variables { get; } = new();

    /// <summary>
    ///     block 覆盖表（子模板覆盖父模板的 block）
    /// </summary>
    public Dictionary<string, List<DejaVuTemplateNode>> Blocks { get; } = new();

    /// <summary>
    ///     模板继承栈（用于检测循环继承）
    /// </summary>
    public Stack<string> ExtendsStack { get; } = new();

    /// <summary>
    ///     是否处于模板继承模式
    /// </summary>
    public bool IsExtending { get; set; }

    /// <summary>
    ///     当前 block 的默认内容（供 super() 渲染）
    /// </summary>
    public List<DejaVuTemplateNode>? CurrentBlockDefault { get; set; }

    /// <summary>
    ///     当前 HTML 输出上下文（决定转义策略）
    /// </summary>
    public HtmlOutputContext OutputContext { get; set; } = HtmlOutputContext.HtmlContent;
}

/// <summary>
///     DejaVu 渲染引擎
/// </summary>
public sealed class DejaVuRenderer : ITemplateEngine
{
    private readonly CompiledTemplateCache? _compileCache;
    private readonly ExpressionParser _expressionParser;
    private readonly FilterRegistry _filters;
    private readonly DejaVuParser _parser;
    private readonly TemplateSecurityValidator _security;
    private readonly TemplateManager? _templateManager;
    private RenderContext _renderContext;

    /// <summary>
    ///     创建 DejaVu 渲染引擎
    /// </summary>
    /// <param name="language">模板语言</param>
    /// <param name="templateManager">模板管理器</param>
    /// <param name="compileCache">编译产物缓存</param>
    public DejaVuRenderer(DejaVuLanguage language, TemplateManager? templateManager = null,
        CompiledTemplateCache? compileCache = null)
    {
        _expressionParser = new ExpressionParser();
        _parser = new DejaVuParser(language, null, _expressionParser);
        _filters = new FilterRegistry();
        _security = new TemplateSecurityValidator();
        _templateManager = templateManager;
        _compileCache = compileCache;
        _renderContext = new RenderContext();
    }

    /// <summary>
    ///     创建 DejaVu 渲染引擎
    /// </summary>
    /// <param name="languageName">模板语言名称</param>
    /// <param name="templateManager">模板管理器</param>
    /// <param name="compileCache">编译产物缓存</param>
    public DejaVuRenderer(string languageName, TemplateManager? templateManager = null,
        CompiledTemplateCache? compileCache = null)
    {
        _expressionParser = new ExpressionParser();
        _parser = new DejaVuParser(languageName, null, _expressionParser);
        _filters = new FilterRegistry();
        _security = new TemplateSecurityValidator();
        _templateManager = templateManager;
        _compileCache = compileCache;
        _renderContext = new RenderContext();
    }

    /// <summary>
    ///     渲染模板
    /// </summary>
    /// <param name="template">模板内容</param>
    /// <param name="context">渲染上下文</param>
    /// <param name="templatePath">模板文件路径（若提供则启用编译缓存）</param>
    /// <returns>渲染结果</returns>
    public string Render(string template, IDictionary<string, object> context, string templatePath = "")
    {
        // 只在顶层调用时重置渲染上下文
        if (_renderContext.ExtendsStack.Count == 0) _renderContext = new RenderContext();

        // 合并上下文变量
        foreach (var (key, value) in context) _renderContext.Variables[key] = value;

        IReadOnlyList<DejaVuTemplateNode> nodes;
        CompiledTemplate? compiledTemplate = null;

        if (_compileCache != null && !string.IsNullOrEmpty(templatePath))
        {
            compiledTemplate = _compileCache.GetOrCompile(templatePath, template, _parser);
            nodes = compiledTemplate.Nodes;

            // 如果有预编译的渲染委托且无 extends，直接使用高性能路径
            if (compiledTemplate.RenderFunc != null && !nodes.OfType<DejaVuExtendsNode>().Any())
                return compiledTemplate.RenderFunc(context);
        }
        else
        {
            var parseResult = _parser.Parse(template);
            nodes = parseResult.Nodes;
        }

        // 检查是否有 extends 指令
        var extendsNode = nodes.OfType<DejaVuExtendsNode>().FirstOrDefault();
        if (extendsNode != null && _templateManager != null)
        {
            // 先收集所有 block 定义
            CollectBlocks(nodes);

            // 然后渲染父模板
            return RenderExtendsNode(extendsNode, context);
        }

        // 如果没有 extends，直接渲染所有节点
        return RenderNodes(nodes, context);
    }

    /// <summary>
    ///     收集 block 定义
    /// </summary>
    private void CollectBlocks(IReadOnlyList<DejaVuTemplateNode> nodes)
    {
        foreach (var node in nodes)
            if (node is DejaVuBlockNode blockNode)
                _renderContext.Blocks[blockNode.Name] = blockNode.Children.ToList();
    }

    /// <summary>
    ///     渲染模板节点
    /// </summary>
    private string RenderNodes(IReadOnlyList<DejaVuTemplateNode> nodes, IDictionary<string, object> context)
    {
        var sb = new StringBuilder();

        foreach (var node in nodes) sb.Append(RenderNode(node, context));

        return sb.ToString();
    }

    /// <summary>
    ///     渲染单个节点
    /// </summary>
    private string RenderNode(DejaVuTemplateNode node, IDictionary<string, object> context)
    {
        return node switch
        {
            DejaVuTextNode textNode => textNode.Text,
            DejaVuCodeNode codeNode => EvaluateCode(codeNode.Code, context),
            DejaVuIfNode ifNode => RenderIfNode(ifNode, context),
            DejaVuLoopNode loopNode => RenderLoopNode(loopNode, context),
            DejaVuMatchNode matchNode => RenderMatchNode(matchNode, context),
            DejaVuBlockNode blockNode => RenderBlockNode(blockNode, context),
            DejaVuExtendsNode => string.Empty,
            DejaVuIncludeNode includeNode => RenderIncludeNode(includeNode, context),
            DejaVuLetNode letNode => RenderLetNode(letNode, context),
            DejaVuWithNode withNode => RenderWithNode(withNode, context),
            DejaVuSuperNode => RenderSuperNode(context),
            DejaVuRawNode rawNode => RenderRawNode(rawNode, context),
            _ => string.Empty
        };
    }

    /// <summary>
    ///     渲染 block 节点
    /// </summary>
    private string RenderBlockNode(DejaVuBlockNode blockNode, IDictionary<string, object> context)
    {
        // 存储当前 block 的默认内容（供 super() 使用）
        var previousDefault = _renderContext.CurrentBlockDefault;
        _renderContext.CurrentBlockDefault = blockNode.Children.ToList();

        try
        {
            if (_renderContext.Blocks.TryGetValue(blockNode.Name, out var overriddenBlock))
                return RenderNodes(overriddenBlock, context);

            return RenderNodes(blockNode.Children, context);
        }
        finally
        {
            _renderContext.CurrentBlockDefault = previousDefault;
        }
    }

    /// <summary>
    ///     渲染 let 节点
    /// </summary>
    private string RenderLetNode(DejaVuLetNode letNode, IDictionary<string, object> context)
    {
        var value = EvaluateExpression(letNode.ParsedExpression, letNode.Expression, context);
        var letContext = new Dictionary<string, object>(context)
        {
            [letNode.VariableName] = value
        };

        return RenderNodes(letNode.Children, letContext);
    }

    /// <summary>
    ///     渲染 with 节点
    /// </summary>
    private string RenderWithNode(DejaVuWithNode withNode, IDictionary<string, object> context)
    {
        var obj = EvaluateExpression(withNode.ParsedExpression, withNode.Expression, context);
        var withContext = new Dictionary<string, object>(context);

        // 将对象属性展开到作用域中
        if (obj is IDictionary<string, object> dict)
        {
            foreach (var (key, val) in dict) withContext[key] = val;
        }
        else if (obj != null)
        {
            var type = obj.GetType();
            foreach (var prop in type.GetProperties()) withContext[prop.Name] = prop.GetValue(obj)!;

            foreach (var field in type.GetFields()) withContext[field.Name] = field.GetValue(obj)!;
        }

        return RenderNodes(withNode.Children, withContext);
    }

    /// <summary>
    ///     渲染 super 节点（渲染父模板的 block 默认内容）
    /// </summary>
    private string RenderSuperNode(IDictionary<string, object> context)
    {
        if (_renderContext.CurrentBlockDefault != null) return RenderNodes(_renderContext.CurrentBlockDefault, context);

        return string.Empty;
    }

    /// <summary>
    ///     渲染 extends 节点
    /// </summary>
    private string RenderExtendsNode(DejaVuExtendsNode extendsNode, IDictionary<string, object> context)
    {
        if (_templateManager == null) return string.Empty;

        try
        {
            var parentTemplate = extendsNode.ParentTemplate.Trim('\'', '"');

            // 防止循环继承
            if (_renderContext.ExtendsStack.Contains(parentTemplate))
                throw new TemplateRenderException($"Circular template inheritance detected: {parentTemplate}");

            _renderContext.ExtendsStack.Push(parentTemplate);
            _renderContext.IsExtending = true;

            var template = _templateManager.LoadAsync(parentTemplate).GetAwaiter().GetResult();
            var result = Render(template, context, parentTemplate);

            _renderContext.ExtendsStack.Pop();
            _renderContext.IsExtending = false;

            return result;
        }
        catch (InvalidOperationException ex)
        {
            Console.Error.WriteLine($"模板继承渲染失败：{ex.Message}");
            return string.Empty;
        }
        catch (KeyNotFoundException ex)
        {
            Console.Error.WriteLine($"模板继承中找不到指定块：{ex.Message}");
            return string.Empty;
        }
    }

    /// <summary>
    ///     渲染 include 节点
    /// </summary>
    private string RenderIncludeNode(DejaVuIncludeNode includeNode, IDictionary<string, object> context)
    {
        if (_templateManager == null) return string.Empty;

        try
        {
            var templatePath = includeNode.TemplatePath.Trim('\'', '"');
            var template = _templateManager.LoadAsync(templatePath).GetAwaiter().GetResult();
            return Render(template, context, templatePath);
        }
        catch (FileNotFoundException ex)
        {
            Console.Error.WriteLine($"引入的模板文件未找到：{ex.Message}");
            return string.Empty;
        }
        catch (InvalidOperationException ex)
        {
            Console.Error.WriteLine($"引入模板渲染失败：{ex.Message}");
            return string.Empty;
        }
    }

    /// <summary>
    ///     渲染 if 节点
    /// </summary>
    private string RenderIfNode(DejaVuIfNode ifNode, IDictionary<string, object> context)
    {
        var conditionResult = EvaluateExpression(ifNode.ParsedCondition, ifNode.Condition, context);
        if (ToBoolean(conditionResult)) return RenderNodes(ifNode.Children, context);

        foreach (var elseIfNode in ifNode.ElseIfNodes)
        {
            var elseIfResult = EvaluateExpression(elseIfNode.ParsedCondition, elseIfNode.Condition, context);
            if (ToBoolean(elseIfResult)) return RenderNodes(elseIfNode.Children, context);
        }

        if (ifNode.ElseChildren.Count > 0) return RenderNodes(ifNode.ElseChildren, context);

        return string.Empty;
    }

    /// <summary>
    ///     求值表达式（优先使用预解析 AST）
    /// </summary>
    private object EvaluateExpression(IExpressionNode? parsedAst, string expression,
        IDictionary<string, object> context)
    {
        var evaluator = new ExpressionEvaluator(context.ToDictionary(k => k.Key, k => (object?)k.Value), _filters);

        if (parsedAst != null) return evaluator.Evaluate(parsedAst) ?? string.Empty;

        var ast = _expressionParser.Parse(expression);
        return evaluator.Evaluate(ast) ?? string.Empty;
    }

    /// <summary>
    ///     渲染 loop 节点
    /// </summary>
    private string RenderLoopNode(DejaVuLoopNode loopNode, IDictionary<string, object> context)
    {
        var sb = new StringBuilder();
        var expressionResult = EvaluateExpression(loopNode.ParsedExpression, loopNode.Expression, context);

        if (expressionResult is IEnumerable enumerable)
        {
            var itemName = loopNode.ItemName ?? "item";
            var iteration = 0;
            foreach (var item in enumerable)
            {
                if (!_security.ValidateLoopIteration(iteration))
                    throw new TemplateTimeoutException("Loop iteration limit exceeded");

                var loopContext = new Dictionary<string, object>(context)
                {
                    [itemName] = item,
                    ["index"] = iteration
                };
                sb.Append(RenderNodes(loopNode.Children, loopContext));
                iteration++;
            }
        }

        return sb.ToString();
    }

    /// <summary>
    ///     渲染 match 节点
    /// </summary>
    private string RenderMatchNode(DejaVuMatchNode matchNode, IDictionary<string, object> context)
    {
        var expressionResult = EvaluateExpression(matchNode.ParsedExpression, matchNode.Expression, context);
        var expressionStr = expressionResult?.ToString() ?? "";

        foreach (var child in matchNode.Children)
            if (child is DejaVuIfNode ifNode)
            {
                var conditionResult = EvaluateExpression(ifNode.ParsedCondition, ifNode.Condition, context);
                if (AreEqual(expressionResult, conditionResult)) return RenderNodes(ifNode.Children, context);

                foreach (var elseIfNode in ifNode.ElseIfNodes)
                {
                    var elseIfResult = EvaluateExpression(elseIfNode.Condition, context);
                    if (AreEqual(expressionResult, elseIfResult)) return RenderNodes(elseIfNode.Children, context);
                }

                if (ifNode.ElseChildren.Count > 0) return RenderNodes(ifNode.ElseChildren, context);
            }
            else if (child is DejaVuTextNode)
            {
            }
            else
            {
                return RenderNode(child, context);
            }

        return string.Empty;
    }

    private static bool AreEqual(object? a, object? b)
    {
        if (a == null && b == null) return true;
        if (a == null || b == null) return false;

        if (a is string sa && b is string sb) return sa == sb;

        if (a is bool ba && b is bool bb) return ba == bb;

        if (a is IConvertible && b is IConvertible)
            try
            {
                return Convert.ToDouble(a) == Convert.ToDouble(b);
            }
            catch (FormatException)
            {
                return a.ToString() == b.ToString();
            }
            catch (InvalidCastException)
            {
                return a.ToString() == b.ToString();
            }

        return a.Equals(b);
    }

    /// <summary>
    ///     评估表达式
    /// </summary>
    private object EvaluateExpression(string expression, IDictionary<string, object> context)
    {
        var ast = _expressionParser.Parse(expression);
        var evaluator = new ExpressionEvaluator(context.ToDictionary(k => k.Key, k => (object?)k.Value), _filters);
        return evaluator.Evaluate(ast) ?? string.Empty;
    }

    /// <summary>
    ///     评估代码——根据输出上下文自动转义
    /// </summary>
    private string EvaluateCode(string code, IDictionary<string, object> context)
    {
        var raw = EvaluateExpression(code, context)?.ToString() ?? string.Empty;
        return EscapeForContext(raw, _renderContext.OutputContext);
    }

    /// <summary>
    ///     根据输出上下文执行转义
    /// </summary>
    private static string EscapeForContext(string input, HtmlOutputContext context)
    {
        return context switch
        {
            HtmlOutputContext.HtmlContent => HtmlEscaper.EscapeHtmlContent(input),
            HtmlOutputContext.HtmlAttribute => HtmlEscaper.EscapeHtmlAttribute(input),
            HtmlOutputContext.JavaScript => HtmlEscaper.EscapeJavaScript(input),
            HtmlOutputContext.Url => HtmlEscaper.EscapeUrl(input),
            HtmlOutputContext.Css => HtmlEscaper.EscapeCss(input),
            HtmlOutputContext.Raw => input,
            _ => HtmlEscaper.EscapeHtmlContent(input)
        };
    }

    /// <summary>
    ///     渲染 raw 节点（原始 HTML 输出，不转义）
    /// </summary>
    private string RenderRawNode(DejaVuRawNode rawNode, IDictionary<string, object> context)
    {
        var previousContext = _renderContext.OutputContext;
        _renderContext.OutputContext = HtmlOutputContext.Raw;

        try
        {
            return RenderNodes(rawNode.Children, context);
        }
        finally
        {
            _renderContext.OutputContext = previousContext;
        }
    }

    private bool ToBoolean(object? value)
    {
        if (value is bool b) return b;
        if (value is null) return false;
        return true;
    }
}