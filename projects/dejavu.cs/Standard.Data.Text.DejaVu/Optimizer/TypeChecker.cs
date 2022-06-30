using Std.Data.Text.DejaVu.Expressions;
using Std.Data.Text.DejaVu.Filters;
using Std.Data.Text.Diagnostics;

namespace Std.Data.Text.DejaVu.Optimizer;

/// <summary>
///     编译期类型检查器——变量类型推导、未定义变量检测、属性访问验证、过滤器类型匹配。
/// </summary>
public sealed class TypeChecker
{
    private readonly DiagnosticSink _diagnostics;
    private readonly FilterRegistry? _filters;


    /// <summary>
    ///     创建类型检查器
    /// </summary>
    /// <param name="diagnostics">诊断消息收集器。</param>
    /// <param name="filters">过滤器注册表（用于过滤器类型检查）。</param>
    public TypeChecker(DiagnosticSink diagnostics, FilterRegistry? filters = null)
    {
        _diagnostics = diagnostics;
        _filters = filters;
    }


    /// <summary>
    ///     对模板节点执行类型检查
    /// </summary>
    /// <param name="nodes">优化后的模板节点。</param>
    /// <param name="knownTypes">已知变量类型（从外部 Data 类型注解提供）。</param>
    /// <returns>推导出的变量类型表。</returns>
    public Dictionary<string, TemplateType> check(IReadOnlyList<DejaVuTemplateNode> nodes,
        Dictionary<string, TemplateType>? knownTypes = null)
    {
        var typeEnv = new TypeEnvironment(knownTypes);

        check_nodes(nodes, typeEnv);

        return typeEnv.inferred_types;
    }

    private void check_nodes(IReadOnlyList<DejaVuTemplateNode> nodes, TypeEnvironment env)
    {
        foreach (var node in nodes) check_node(node, env);
    }

    private void check_node(DejaVuTemplateNode node, TypeEnvironment env)
    {
        switch (node)
        {
            case DejaVuCodeNode codeNode:
                check_expression(codeNode.parsed_expression, codeNode.code, env);
                break;
            case DejaVuIfNode ifNode:
                var condType = check_expression(ifNode.parsed_condition, ifNode.condition, env);
                if (condType != TemplateType.unknown && condType != TemplateType.boolean)
                    _diagnostics.report_warning(string.Empty, default, "NonBooleanCondition",
                        $"if 条件表达式类型为 {condType}，期望 Boolean");

                check_nodes(ifNode.children, env);
                foreach (var elseIf in ifNode.else_if_nodes)
                {
                    check_expression(elseIf.parsed_condition, elseIf.condition, env);
                    check_nodes(elseIf.children, env);
                }

                check_nodes(ifNode.else_children, env);
                break;
            case DejaVuLoopNode loopNode:
                var iterType = check_expression(loopNode.parsed_expression, loopNode.expression, env);
                if (iterType != TemplateType.unknown && iterType != TemplateType.array &&
                    iterType != TemplateType.@object)
                    _diagnostics.report_warning(string.Empty, default, "NonIterableLoop",
                        $"loop 表达式类型为 {iterType}，期望 Array 或 Object");

                var itemName = loopNode.item_name ?? "item";
                var itemType = iterType == TemplateType.array ? TemplateType.any : TemplateType.unknown;
                env.push_scope();
                env.declare(itemName, itemType);
                env.declare("index", TemplateType.number);
                check_nodes(loopNode.children, env);
                env.pop_scope();
                break;
            case DejaVuLetNode letNode:
                var letType = check_expression(letNode.parsed_expression, letNode.expression, env);
                env.push_scope();
                env.declare(letNode.variable_name, letType);
                check_nodes(letNode.children, env);
                env.pop_scope();
                break;
            case DejaVuWithNode withNode:
                check_expression(withNode.parsed_expression, withNode.expression, env);
                env.push_scope();
                if (!string.IsNullOrEmpty(withNode.alias_name)) env.declare(withNode.alias_name, TemplateType.@object);

                check_nodes(withNode.children, env);
                env.pop_scope();
                break;
            case DejaVuBlockNode blockNode:
                check_nodes(blockNode.children, env);
                break;
            case DejaVuMatchNode matchNode:
                check_expression(matchNode.parsed_expression, matchNode.expression, env);
                check_nodes(matchNode.children, env);
                break;
            case DejaVuRawNode rawNode:
                check_nodes(rawNode.children, env);
                break;
        }
    }

    private TemplateType check_expression(IExpressionNode? parsedAst, string fallback, TypeEnvironment env)
    {
        if (parsedAst != null) return check_expression_node(parsedAst, env);

        return TemplateType.unknown;
    }

    private TemplateType check_expression_node(IExpressionNode node, TypeEnvironment env)
    {
        return node switch
        {
            LiteralNode lit => check_literal(lit),
            IdentifierNode id => check_identifier(id, env),
            BinaryNode binary => check_binary(binary, env),
            UnaryNode unary => check_unary(unary, env),
            MemberAccessNode member => check_member_access(member, env),
            CallNode call => check_call(call, env),
            IndexNode index => check_index(index, env),
            PipeNode pipe => check_pipe(pipe, env),
            _ => TemplateType.unknown
        };
    }

    private TemplateType check_literal(LiteralNode lit)
    {
        return lit.value switch
        {
            null => TemplateType.@null,
            bool => TemplateType.boolean,
            double => TemplateType.number,
            string => TemplateType.@string,
            _ => TemplateType.unknown
        };
    }

    private TemplateType check_identifier(IdentifierNode id, TypeEnvironment env)
    {
        if (env.try_get_type(id.name, out var type)) return type;

        _diagnostics.report_warning(string.Empty, default, "UndefinedVariable",
            $"未定义的变量 \"{id.name}\"，运行期将从模板上下文中解析");

        env.infer_type(id.name, TemplateType.any);
        return TemplateType.any;
    }

    private TemplateType check_binary(BinaryNode binary, TypeEnvironment env)
    {
        var leftType = check_expression_node(binary.left, env);
        var rightType = check_expression_node(binary.right, env);

        return binary.@operator switch
        {
            BinaryOperator.add => infer_add_type(leftType, rightType),
            BinaryOperator.subtract or BinaryOperator.multiply or BinaryOperator.divide or BinaryOperator.modulo
                => TemplateType.number,
            BinaryOperator.equal or BinaryOperator.not_equal => TemplateType.boolean,
            BinaryOperator.less_than or BinaryOperator.less_than_or_equal or BinaryOperator.greater_than
                or BinaryOperator.greater_than_or_equal
                => TemplateType.boolean,
            BinaryOperator.and or BinaryOperator.or => TemplateType.boolean,
            _ => TemplateType.unknown
        };
    }

    private TemplateType check_unary(UnaryNode unary, TypeEnvironment env)
    {
        var operandType = check_expression_node(unary.operand, env);

        return unary.@operator switch
        {
            UnaryOperator.negate => operandType == TemplateType.number ? TemplateType.number : TemplateType.unknown,
            UnaryOperator.not => TemplateType.boolean,
            _ => TemplateType.unknown
        };
    }

    private TemplateType check_member_access(MemberAccessNode member, TypeEnvironment env)
    {
        var objType = check_expression_node(member.@object, env);

        if (objType is TemplateType.unknown or TemplateType.any) return TemplateType.any;

        return TemplateType.any;
    }

    private TemplateType check_call(CallNode call, TypeEnvironment env)
    {
        check_expression_node(call.function, env);
        foreach (var arg in call.arguments) check_expression_node(arg, env);

        return TemplateType.any;
    }

    private TemplateType check_index(IndexNode index, TypeEnvironment env)
    {
        var objType = check_expression_node(index.@object, env);
        check_expression_node(index.index, env);

        if (objType == TemplateType.array) return TemplateType.any;

        return TemplateType.any;
    }

    private TemplateType check_pipe(PipeNode pipe, TypeEnvironment env)
    {
        var inputType = check_expression_node(pipe.left, env);
        foreach (var arg in pipe.arguments) check_expression_node(arg, env);

        if (_filters != null && !_filters.has_filter(pipe.filter_name))
            _diagnostics.report_warning(string.Empty, default, "UnknownFilter",
                $"未知的过滤器 \"{pipe.filter_name}\"");

        return infer_filter_output_type(pipe.filter_name, inputType);
    }

    private static TemplateType infer_add_type(TemplateType left, TemplateType right)
    {
        if (left == TemplateType.@string || right == TemplateType.@string) return TemplateType.@string;

        if (left == TemplateType.number && right == TemplateType.number) return TemplateType.number;

        return TemplateType.unknown;
    }

    private static TemplateType infer_filter_output_type(string filterName, TemplateType inputType)
    {
        return filterName switch
        {
            "uppercase" or "lowercase" or "trim" or "capitalize" or "strip_html" or "escape" or "newline_to_br"
                => TemplateType.@string,
            "truncate" => TemplateType.@string,
            "length" => TemplateType.number,
            "first" or "last" => TemplateType.any,
            "reverse" or "sort" => inputType == TemplateType.array ? TemplateType.array : TemplateType.@string,
            "join" => TemplateType.@string,
            "default" => inputType,
            _ => TemplateType.any
        };
    }
}