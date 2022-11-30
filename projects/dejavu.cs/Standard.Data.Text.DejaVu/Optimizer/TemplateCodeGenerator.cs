using System.Collections;
using System.Linq.Expressions;
using System.Reflection;
using System.Text;
using Std.Data.Text.DejaVu.Expressions;
using Std.Data.Text.DejaVu.Security;

namespace Std.Data.Text.DejaVu.Optimizer;

/// <summary>
///     模板代码生成器——将 AST 编译为可缓存的渲染委托。
///     使用 System.Linq.Expressions 构建表达式树，JIT 编译为原生代码。
/// </summary>
public sealed class TemplateCodeGenerator
{
    /// <summary>
    ///     将优化后的模板节点编译为渲染委托
    /// </summary>
    /// <param name="nodes">优化后的模板节点列表。</param>
    /// <returns>渲染委托：输入上下文变量 → 输出渲染字符串。</returns>
    public Func<IDictionary<string, object>, string> compile(IReadOnlyList<DejaVuTemplateNode> nodes)
    {
        var contextParam = Expression.Parameter(typeof(IDictionary<string, object>), "ctx");
        var sbVar = Expression.Variable(typeof(StringBuilder), "sb");

        var expressions = new List<Expression>
        {
            Expression.Assign(sbVar, Expression.New(typeof(StringBuilder)))
        };

        foreach (var node in nodes) expressions.Add(generate_node(node, contextParam, sbVar));

        expressions.Add(Expression.Call(
            sbVar,
            typeof(StringBuilder).GetMethod("ToString", Type.EmptyTypes)!
        ));

        var body = Expression.Block(
            [sbVar],
            expressions
        );

        var lambda = Expression.Lambda<Func<IDictionary<string, object>, string>>(
            body,
            contextParam
        );

        return lambda.Compile();
    }

    private Expression generate_node(DejaVuTemplateNode node, ParameterExpression ctx, Expression sb)
    {
        return node switch
        {
            DejaVuTextNode textNode => generate_text_node(textNode, sb),
            DejaVuCodeNode codeNode => generate_code_node(codeNode, ctx, sb),
            DejaVuIfNode ifNode => generate_if_node(ifNode, ctx, sb),
            DejaVuLoopNode loopNode => generate_loop_node(loopNode, ctx, sb),
            DejaVuLetNode letNode => generate_let_node(letNode, ctx, sb),
            DejaVuRawNode rawNode => generate_raw_node(rawNode, ctx, sb),
            _ => Expression.Empty()
        };
    }

    private Expression generate_text_node(DejaVuTextNode textNode, Expression sb)
    {
        return Expression.Call(
            sb,
            typeof(StringBuilder).GetMethod("Append", [typeof(string)])!,
            Expression.Constant(textNode.text)
        );
    }

    private Expression generate_code_node(DejaVuCodeNode codeNode, ParameterExpression ctx, Expression sb)
    {
        var evalExpr = generate_expression_eval(codeNode.parsed_expression, codeNode.code, ctx);
        var escapedExpr = Expression.Call(
            typeof(HtmlEscaper).GetMethod("EscapeHtmlContent", [typeof(string)])!,
            evalExpr
        );

        return Expression.Call(
            sb,
            typeof(StringBuilder).GetMethod("Append", [typeof(string)])!,
            escapedExpr
        );
    }

    private Expression generate_if_node(DejaVuIfNode ifNode, ParameterExpression ctx, Expression sb)
    {
        var conditionExpr = generate_expression_eval(ifNode.parsed_condition, ifNode.condition, ctx);
        var toBoolCall = Expression.Call(
            typeof(TemplateCodeGenerator).GetMethod(nameof(to_boolean), BindingFlags.NonPublic | BindingFlags.Static)!,
            conditionExpr
        );

        var thenExpr = generate_nodes_block(ifNode.children, ctx, sb);
        var elseExpr = ifNode.else_children.Count > 0
            ? generate_nodes_block(ifNode.else_children, ctx, sb)
            : Expression.Empty();

        Expression result = Expression.IfThenElse(toBoolCall, thenExpr, elseExpr);

        foreach (var elseIfNode in ifNode.else_if_nodes.AsEnumerable().Reverse())
        {
            var elseIfCondition = generate_expression_eval(elseIfNode.parsed_condition, elseIfNode.condition, ctx);
            var elseIfBool = Expression.Call(
                typeof(TemplateCodeGenerator).GetMethod(nameof(to_boolean),
                    BindingFlags.NonPublic | BindingFlags.Static)!,
                elseIfCondition
            );
            var elseIfBody = generate_nodes_block(elseIfNode.children, ctx, sb);
            result = Expression.IfThenElse(elseIfBool, elseIfBody, result);
        }

        return result;
    }

    private Expression generate_loop_node(DejaVuLoopNode loopNode, ParameterExpression ctx, Expression sb)
    {
        var itemName = loopNode.item_name ?? "item";
        var iterableExpr = generate_expression_eval(loopNode.parsed_expression, loopNode.expression, ctx);

        return Expression.Call(
            typeof(TemplateCodeGenerator).GetMethod(nameof(execute_loop),
                BindingFlags.NonPublic | BindingFlags.Static)!,
            ctx,
            iterableExpr,
            Expression.Constant(itemName),
            Expression.Constant(loopNode.children.ToArray()),
            Expression.Constant(this)
        );
    }

    private Expression generate_let_node(DejaVuLetNode letNode, ParameterExpression ctx, Expression sb)
    {
        var valueExpr = generate_expression_eval(letNode.parsed_expression, letNode.expression, ctx);

        return Expression.Call(
            typeof(TemplateCodeGenerator).GetMethod(nameof(execute_let), BindingFlags.NonPublic | BindingFlags.Static)!,
            ctx,
            Expression.Constant(letNode.variable_name),
            valueExpr,
            Expression.Constant(letNode.children.ToArray()),
            Expression.Constant(this)
        );
    }

    private Expression generate_raw_node(DejaVuRawNode rawNode, ParameterExpression ctx, Expression sb)
    {
        return Expression.Call(
            typeof(TemplateCodeGenerator).GetMethod(nameof(execute_raw), BindingFlags.NonPublic | BindingFlags.Static)!,
            ctx,
            Expression.Constant(rawNode.children.ToArray()),
            Expression.Constant(this)
        );
    }

    private Expression generate_nodes_block(IReadOnlyList<DejaVuTemplateNode> nodes, ParameterExpression ctx,
        Expression sb)
    {
        if (nodes.Count == 0) return Expression.Empty();

        var expressions = new List<Expression>();
        foreach (var node in nodes) expressions.Add(generate_node(node, ctx, sb));

        return Expression.Block(expressions);
    }

    private Expression generate_expression_eval(IExpressionNode? parsedAst, string fallbackExpression,
        ParameterExpression ctx)
    {
        if (parsedAst != null)
            return Expression.Call(
                typeof(TemplateCodeGenerator).GetMethod(nameof(evaluate_node),
                    BindingFlags.NonPublic | BindingFlags.Static)!,
                Expression.Constant(parsedAst),
                ctx
            );

        return Expression.Call(
            typeof(TemplateCodeGenerator).GetMethod(nameof(evaluate_fallback),
                BindingFlags.NonPublic | BindingFlags.Static)!,
            Expression.Constant(fallbackExpression),
            ctx
        );
    }

    #region 运行时辅助方法

    private static bool to_boolean(object? value)
    {
        if (value is bool b) return b;

        if (value is null) return false;

        return true;
    }

    private static void execute_loop(
        IDictionary<string, object> ctx,
        object? iterable,
        string itemName,
        DejaVuTemplateNode[] children,
        TemplateCodeGenerator generator)
    {
        if (iterable is not IEnumerable enumerable) return;

        var sb = new StringBuilder();
        var index = 0;
        foreach (var item in enumerable)
        {
            var loopCtx = new Dictionary<string, object>(ctx)
            {
                [itemName] = item,
                ["index"] = index
            };

            var renderFunc = generator.compile(children);
            sb.Append(renderFunc(loopCtx));
            index++;
        }
    }

    private static void execute_let(
        IDictionary<string, object> ctx,
        string variableName,
        object? value,
        DejaVuTemplateNode[] children,
        TemplateCodeGenerator generator)
    {
        var letCtx = new Dictionary<string, object>(ctx)
        {
            [variableName] = value!
        };

        var renderFunc = generator.compile(children);
        var sb = new StringBuilder();
        sb.Append(renderFunc(letCtx));
    }

    private static void execute_raw(
        IDictionary<string, object> ctx,
        DejaVuTemplateNode[] children,
        TemplateCodeGenerator generator)
    {
        var renderFunc = generator.compile(children);
        var sb = new StringBuilder();
        sb.Append(renderFunc(ctx));
    }

    private static object? evaluate_node(IExpressionNode node, IDictionary<string, object> ctx)
    {
        var evaluator = new ExpressionEvaluator(ctx.ToDictionary(k => k.Key, k => (object?)k.Value));
        return evaluator.evaluate(node);
    }

    private static object? evaluate_fallback(string expression, IDictionary<string, object> ctx)
    {
        var parser = new ExpressionParser();
        var ast = parser.parse(expression);
        var evaluator = new ExpressionEvaluator(ctx.ToDictionary(k => k.Key, k => (object?)k.Value));
        return evaluator.evaluate(ast);
    }

    #endregion
}