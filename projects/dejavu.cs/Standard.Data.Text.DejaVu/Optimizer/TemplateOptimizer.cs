using System.Text;
using Std.Data.Text.DejaVu.Expressions;

namespace Std.Data.Text.DejaVu.Optimizer;

/// <summary>
///     模板优化器——死代码消除、节点合并
/// </summary>
public sealed class TemplateOptimizer
{
    /// <summary>
    ///     优化模板 AST
    /// </summary>
    public List<DejaVuTemplateNode> optimize(List<DejaVuTemplateNode> nodes)
    {
        var optimized = new List<DejaVuTemplateNode>();

        foreach (var node in nodes)
        {
            var optimizedNode = optimize_node(node);
            if (optimizedNode != null)
            {
                if (optimizedNode is IEnumerable<DejaVuTemplateNode> flattened)
                    optimized.AddRange(flattened);
                else
                    optimized.Add(optimizedNode);
            }
        }

        return merge_consecutive_text_nodes(optimized);
    }


    /// <summary>
    ///     优化单个节点
    /// </summary>
    private DejaVuTemplateNode? optimize_node(DejaVuTemplateNode node)
    {
        return node switch
        {
            DejaVuIfNode ifNode => optimize_if_node(ifNode),
            DejaVuLoopNode loopNode => optimize_loop_node(loopNode),
            DejaVuBlockNode blockNode => optimize_block_node(blockNode),
            DejaVuLetNode letNode => optimize_let_node(letNode),
            DejaVuWithNode withNode => optimize_with_node(withNode),
            DejaVuRawNode rawNode => optimize_raw_node(rawNode),
            DejaVuCodeNode codeNode => optimize_code_node(codeNode),
            _ => node
        };
    }


    /// <summary>
    ///     优化 if 节点——永假分支消除
    /// </summary>
    private DejaVuTemplateNode? optimize_if_node(DejaVuIfNode ifNode)
    {
        var optimizedCondition = optimize_expression(ifNode.parsed_condition);

        // 常量折叠后的条件检查
        if (is_constant_false(optimizedCondition))
        {
            // 条件恒为 false，跳过 if 体，检查 else if / else
            foreach (var elseIfNode in ifNode.else_if_nodes)
            {
                var optimizedElseIf = optimize_expression(elseIfNode.parsed_condition);
                if (!is_constant_false(optimizedElseIf))
                    return new DejaVuCodeNode
                    {
                        code = elseIfNode.condition,
                        parsed_expression = optimize_expression(optimizedElseIf)
                    };
            }

            if (ifNode.else_children.Count > 0)
            {
                var optimizedElse = optimize(ifNode.else_children);
                return optimizedElse.Count == 1 ? optimizedElse[0] : null;
            }

            return null; // 整棵 if 树删除
        }

        var children = optimize(ifNode.children);
        var elseChildren = optimize(ifNode.else_children);
        var elseIfNodes = new List<DejaVuElseIfNode>();

        foreach (var elseIfNode in ifNode.else_if_nodes)
        {
            var optimizedElseIf = optimize_expression(elseIfNode.parsed_condition);
            if (!is_constant_false(optimizedElseIf))
                elseIfNodes.Add(new DejaVuElseIfNode
                {
                    condition = elseIfNode.condition,
                    parsed_condition = optimizedElseIf,
                    children = optimize(elseIfNode.children)
                });
        }

        return new DejaVuIfNode
        {
            condition = ifNode.condition,
            parsed_condition = optimize_expression(optimizedCondition),
            children = children,
            else_children = elseChildren,
            else_if_nodes = elseIfNodes
        };
    }


    /// <summary>
    ///     优化 loop 节点
    /// </summary>
    private DejaVuTemplateNode optimize_loop_node(DejaVuLoopNode loopNode)
    {
        return new DejaVuLoopNode
        {
            expression = loopNode.expression,
            parsed_expression = optimize_expression(loopNode.parsed_expression),
            item_name = loopNode.item_name,
            children = optimize(loopNode.children)
        };
    }


    /// <summary>
    ///     优化 block 节点
    /// </summary>
    private DejaVuTemplateNode optimize_block_node(DejaVuBlockNode blockNode)
    {
        return new DejaVuBlockNode
        {
            name = blockNode.name,
            children = optimize(blockNode.children)
        };
    }


    /// <summary>
    ///     优化 let 节点
    /// </summary>
    private DejaVuTemplateNode optimize_let_node(DejaVuLetNode letNode)
    {
        return new DejaVuLetNode
        {
            variable_name = letNode.variable_name,
            expression = letNode.expression,
            parsed_expression = optimize_expression(letNode.parsed_expression),
            children = optimize(letNode.children)
        };
    }


    /// <summary>
    ///     优化 with 节点
    /// </summary>
    private DejaVuTemplateNode optimize_with_node(DejaVuWithNode withNode)
    {
        return new DejaVuWithNode
        {
            alias_name = withNode.alias_name,
            expression = withNode.expression,
            parsed_expression = optimize_expression(withNode.parsed_expression),
            children = optimize(withNode.children)
        };
    }


    /// <summary>
    ///     优化 raw 节点
    /// </summary>
    private DejaVuTemplateNode optimize_raw_node(DejaVuRawNode rawNode)
    {
        return new DejaVuRawNode { children = optimize(rawNode.children) };
    }


    /// <summary>
    ///     优化 code 节点
    /// </summary>
    private DejaVuTemplateNode optimize_code_node(DejaVuCodeNode codeNode)
    {
        var optimized = optimize_expression(codeNode.parsed_expression);
        if (optimized is LiteralNode lit) return new DejaVuTextNode { text = lit.value?.ToString() ?? string.Empty };

        return new DejaVuCodeNode { code = codeNode.code, parsed_expression = optimized };
    }


    /// <summary>
    ///     判断表达式是否为常量 false
    /// </summary>
    private static bool is_constant_false(IExpressionNode? node)
    {
        return node is LiteralNode { value: false };
    }


    /// <summary>
    ///     判断表达式是否为常量 true
    /// </summary>
    private static bool is_constant_true(IExpressionNode? node)
    {
        return node is LiteralNode { value: true };
    }


    /// <summary>
    ///     合并连续文本节点
    /// </summary>
    private static List<DejaVuTemplateNode> merge_consecutive_text_nodes(List<DejaVuTemplateNode> nodes)
    {
        if (nodes.Count < 2) return nodes;

        var result = new List<DejaVuTemplateNode>();
        var sb = new StringBuilder();

        foreach (var node in nodes)
            if (node is DejaVuTextNode textNode)
            {
                sb.Append(textNode.text);
            }
            else
            {
                if (sb.Length > 0)
                {
                    result.Add(new DejaVuTextNode { text = sb.ToString() });
                    sb.Clear();
                }

                result.Add(node);
            }

        if (sb.Length > 0) result.Add(new DejaVuTextNode { text = sb.ToString() });

        return result;
    }


    /// <summary>
    ///     优化表达式（常量折叠包装）
    /// </summary>
    private static IExpressionNode? optimize_expression(IExpressionNode? node)
    {
        return node == null ? null : ExpressionOptimizer.optimize(node);
    }
}