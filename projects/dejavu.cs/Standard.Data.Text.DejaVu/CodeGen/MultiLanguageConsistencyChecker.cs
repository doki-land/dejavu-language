using Std.Data.Text.DejaVu.Expressions;
using Std.Data.Text.DejaVu.Optimizer;

namespace Std.Data.Text.DejaVu.CodeGen;

/// <summary>
///     多语言一致性验证器——验证同一模板生成的 C#/TypeScript/Java 代码结构一致性。
///     通过比较 AST 节点覆盖率、过滤器覆盖率和表达式类型覆盖率来评估一致性。
/// </summary>
public sealed class MultiLanguageConsistencyChecker
{
    /// <summary>
    ///     验证模板的多语言一致性
    /// </summary>
    /// <param name="source">模板源码。</param>
    /// <param name="compiler">编译器。</param>
    /// <returns>一致性检查结果。</returns>
    public ConsistencyResult check(string source, DejaVuCompiler compiler)
    {
        var parseResult = new DejaVuParser("doki").parse(source);
        var optimizer = new TemplateOptimizer();
        var optimizedNodes = optimizer.optimize([.. parseResult.nodes]);

        var csharpOutput = compiler.compile(source).render_func != null
            ? "JIT compiled"
            : "Interpreter mode";

        var tsOutput = compiler.compile_to_type_script(source);
        var javaOutput = compiler.compile_to_java(source);

        var nodeTypes = collect_node_types(optimizedNodes);
        var expressionTypes = collect_expression_types(optimizedNodes);
        var filters = collect_filters(optimizedNodes);

        var tsNodeCoverage = check_node_type_coverage(nodeTypes, tsOutput, "TypeScript");
        var javaNodeCoverage = check_node_type_coverage(nodeTypes, javaOutput, "Java");

        var tsExprCoverage = check_expression_coverage(expressionTypes, tsOutput);
        var javaExprCoverage = check_expression_coverage(expressionTypes, javaOutput);

        var tsFilterCoverage = check_filter_coverage(filters, tsOutput);
        var javaFilterCoverage = check_filter_coverage(filters, javaOutput);

        var astCoverage = (tsNodeCoverage + javaNodeCoverage) / 2.0;
        var filterCoverage = (tsFilterCoverage + javaFilterCoverage) / 2.0;
        var exprCoverage = (tsExprCoverage + javaExprCoverage) / 2.0;

        var uncoveredNodes = new List<string>();
        foreach (var nodeType in nodeTypes)
            if (!tsOutput.Contains(nodeType, StringComparison.OrdinalIgnoreCase) &&
                !javaOutput.Contains(nodeType, StringComparison.OrdinalIgnoreCase))
                uncoveredNodes.Add(nodeType);

        return new ConsistencyResult
        {
            template_source = source,
            c_sharp_output = csharpOutput,
            type_script_output = tsOutput,
            java_output = javaOutput,
            ast_coverage = astCoverage,
            filter_coverage = filterCoverage,
            expression_coverage = exprCoverage,
            uncovered_node_types = uncoveredNodes,
            uncovered_filters =
            [
                .. filters.Where(f =>
                    !tsOutput.Contains(f, StringComparison.OrdinalIgnoreCase) &&
                    !javaOutput.Contains(f, StringComparison.OrdinalIgnoreCase))
            ],
            uncovered_expression_types =
            [
                .. expressionTypes.Where(e =>
                    !tsOutput.Contains(e, StringComparison.OrdinalIgnoreCase) &&
                    !javaOutput.Contains(e, StringComparison.OrdinalIgnoreCase))
            ]
        };
    }

    private static HashSet<string> collect_node_types(IReadOnlyList<DejaVuTemplateNode> nodes)
    {
        var types = new HashSet<string>();

        foreach (var node in nodes)
        {
            types.Add(node.node_type.ToString());

            switch (node)
            {
                case DejaVuIfNode ifNode:
                    collect_node_types(ifNode.children);
                    foreach (var elseIf in ifNode.else_if_nodes)
                    {
                        types.Add("ElseIf");
                        collect_node_types(elseIf.children);
                    }

                    collect_node_types(ifNode.else_children);
                    break;
                case DejaVuLoopNode loopNode:
                    collect_node_types(loopNode.children);
                    break;
                case DejaVuLetNode letNode:
                    collect_node_types(letNode.children);
                    break;
                case DejaVuWithNode withNode:
                    collect_node_types(withNode.children);
                    break;
                case DejaVuBlockNode blockNode:
                    collect_node_types(blockNode.children);
                    break;
                case DejaVuRawNode rawNode:
                    collect_node_types(rawNode.children);
                    break;
                case DejaVuMatchNode matchNode:
                    collect_node_types(matchNode.children);
                    break;
            }
        }

        return types;
    }

    private static HashSet<string> collect_expression_types(IReadOnlyList<DejaVuTemplateNode> nodes)
    {
        var types = new HashSet<string>();

        foreach (var node in nodes)
            switch (node)
            {
                case DejaVuCodeNode codeNode:
                    collect_expr_types(codeNode.parsed_expression, types);
                    break;
                case DejaVuIfNode ifNode:
                    collect_expr_types(ifNode.parsed_condition, types);
                    collect_expression_types(ifNode.children);
                    foreach (var elseIf in ifNode.else_if_nodes)
                    {
                        collect_expr_types(elseIf.parsed_condition, types);
                        collect_expression_types(elseIf.children);
                    }

                    collect_expression_types(ifNode.else_children);
                    break;
                case DejaVuLoopNode loopNode:
                    collect_expr_types(loopNode.parsed_expression, types);
                    collect_expression_types(loopNode.children);
                    break;
                case DejaVuLetNode letNode:
                    collect_expr_types(letNode.parsed_expression, types);
                    collect_expression_types(letNode.children);
                    break;
                case DejaVuWithNode withNode:
                    collect_expr_types(withNode.parsed_expression, types);
                    collect_expression_types(withNode.children);
                    break;
                case DejaVuMatchNode matchNode:
                    collect_expr_types(matchNode.parsed_expression, types);
                    collect_expression_types(matchNode.children);
                    break;
                case DejaVuRawNode rawNode:
                    collect_expression_types(rawNode.children);
                    break;
            }

        return types;
    }

    private static void collect_expr_types(IExpressionNode? node, HashSet<string> types)
    {
        if (node == null) return;

        types.Add(node.GetType().Name);

        switch (node)
        {
            case BinaryNode binary:
                collect_expr_types(binary.left, types);
                collect_expr_types(binary.right, types);
                break;
            case UnaryNode unary:
                collect_expr_types(unary.operand, types);
                break;
            case MemberAccessNode member:
                collect_expr_types(member.@object, types);
                break;
            case CallNode call:
                collect_expr_types(call.function, types);
                foreach (var arg in call.arguments) collect_expr_types(arg, types);

                break;
            case IndexNode index:
                collect_expr_types(index.@object, types);
                collect_expr_types(index.index, types);
                break;
            case PipeNode pipe:
                collect_expr_types(pipe.left, types);
                foreach (var arg in pipe.arguments) collect_expr_types(arg, types);

                break;
        }
    }

    private static HashSet<string> collect_filters(IReadOnlyList<DejaVuTemplateNode> nodes)
    {
        var filters = new HashSet<string>();
        collect_filters_from_nodes(nodes, filters);
        return filters;
    }

    private static void collect_filters_from_nodes(IReadOnlyList<DejaVuTemplateNode> nodes, HashSet<string> filters)
    {
        foreach (var node in nodes)
            switch (node)
            {
                case DejaVuCodeNode codeNode:
                    collect_filters_from_expr(codeNode.parsed_expression, filters);
                    break;
                case DejaVuIfNode ifNode:
                    collect_filters_from_expr(ifNode.parsed_condition, filters);
                    collect_filters_from_nodes(ifNode.children, filters);
                    foreach (var elseIf in ifNode.else_if_nodes)
                    {
                        collect_filters_from_expr(elseIf.parsed_condition, filters);
                        collect_filters_from_nodes(elseIf.children, filters);
                    }

                    collect_filters_from_nodes(ifNode.else_children, filters);
                    break;
                case DejaVuLoopNode loopNode:
                    collect_filters_from_expr(loopNode.parsed_expression, filters);
                    collect_filters_from_nodes(loopNode.children, filters);
                    break;
                case DejaVuLetNode letNode:
                    collect_filters_from_expr(letNode.parsed_expression, filters);
                    collect_filters_from_nodes(letNode.children, filters);
                    break;
                case DejaVuWithNode withNode:
                    collect_filters_from_expr(withNode.parsed_expression, filters);
                    collect_filters_from_nodes(withNode.children, filters);
                    break;
                case DejaVuMatchNode matchNode:
                    collect_filters_from_expr(matchNode.parsed_expression, filters);
                    collect_filters_from_nodes(matchNode.children, filters);
                    break;
                case DejaVuRawNode rawNode:
                    collect_filters_from_nodes(rawNode.children, filters);
                    break;
            }
    }

    private static void collect_filters_from_expr(IExpressionNode? node, HashSet<string> filters)
    {
        if (node == null) return;

        if (node is PipeNode pipe)
        {
            filters.Add(pipe.filter_name);
            collect_filters_from_expr(pipe.left, filters);
            foreach (var arg in pipe.arguments) collect_filters_from_expr(arg, filters);
        }
        else if (node is BinaryNode binary)
        {
            collect_filters_from_expr(binary.left, filters);
            collect_filters_from_expr(binary.right, filters);
        }
        else if (node is UnaryNode unary)
        {
            collect_filters_from_expr(unary.operand, filters);
        }
        else if (node is MemberAccessNode member)
        {
            collect_filters_from_expr(member.@object, filters);
        }
        else if (node is CallNode call)
        {
            collect_filters_from_expr(call.function, filters);
            foreach (var arg in call.arguments) collect_filters_from_expr(arg, filters);
        }
    }

    private static double check_node_type_coverage(HashSet<string> nodeTypes, string output, string language)
    {
        if (nodeTypes.Count == 0) return 1.0;

        var covered = 0;
        foreach (var type in nodeTypes)
            if (output.Contains(type, StringComparison.OrdinalIgnoreCase))
                covered++;

        return (double)covered / nodeTypes.Count;
    }

    private static double check_expression_coverage(HashSet<string> expressionTypes, string output)
    {
        if (expressionTypes.Count == 0) return 1.0;

        var covered = 0;
        foreach (var type in expressionTypes)
        {
            var keyword = type.Replace("Node", "");
            if (output.Contains(keyword, StringComparison.OrdinalIgnoreCase)) covered++;
        }

        return (double)covered / expressionTypes.Count;
    }

    private static double check_filter_coverage(HashSet<string> filters, string output)
    {
        if (filters.Count == 0) return 1.0;

        var covered = 0;
        foreach (var filter in filters)
            if (output.Contains($"\"{filter}\"") || output.Contains($"\"{filter}\""))
                covered++;

        return (double)covered / filters.Count;
    }

    /// <summary>
    ///     一致性检查结果
    /// </summary>
    public sealed class ConsistencyResult
    {
        /// <summary>
        ///     模板源码
        /// </summary>
        public string template_source { get; init; } = string.Empty;


        /// <summary>
        ///     C# 渲染输出
        /// </summary>
        public string c_sharp_output { get; init; } = string.Empty;


        /// <summary>
        ///     TypeScript 生成源码
        /// </summary>
        public string type_script_output { get; init; } = string.Empty;


        /// <summary>
        ///     Java 生成源码
        /// </summary>
        public string java_output { get; init; } = string.Empty;


        /// <summary>
        ///     AST 节点覆盖率（0.0-1.0）
        /// </summary>
        public double ast_coverage { get; init; }


        /// <summary>
        ///     过滤器覆盖率（0.0-1.0）
        /// </summary>
        public double filter_coverage { get; init; }


        /// <summary>
        ///     表达式类型覆盖率（0.0-1.0）
        /// </summary>
        public double expression_coverage { get; init; }


        /// <summary>
        ///     总体一致性得分（0.0-1.0）
        /// </summary>
        public double overall_score => (ast_coverage + filter_coverage + expression_coverage) / 3.0;


        /// <summary>
        ///     未覆盖的 AST 节点类型
        /// </summary>
        public List<string> uncovered_node_types { get; init; } = [];


        /// <summary>
        ///     未覆盖的过滤器
        /// </summary>
        public List<string> uncovered_filters { get; init; } = [];


        /// <summary>
        ///     未覆盖的表达式类型
        /// </summary>
        public List<string> uncovered_expression_types { get; init; } = [];
    }
}