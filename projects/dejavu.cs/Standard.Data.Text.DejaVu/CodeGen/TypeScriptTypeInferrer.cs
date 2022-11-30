using System.Text;
using Std.Data.Text.DejaVu.Expressions;
using Std.Data.Text.DejaVu.Optimizer;

namespace Std.Data.Text.DejaVu.CodeGen;

/// <summary>
///     TypeScript 类型推导器——从模板 AST 推导所需的 Data 接口。
///     输出 TypeScript 接口定义，用于编译期类型检查。
/// </summary>
public sealed class TypeScriptTypeInferrer
{
    /// <summary>
    ///     从模板节点推导 TypeScript Data 接口
    /// </summary>
    /// <param name="nodes">优化后的模板节点。</param>
    /// <param name="interfaceName">接口名称。</param>
    /// <returns>TypeScript 接口源码。</returns>
    public string infer_interface(IReadOnlyList<DejaVuTemplateNode> nodes, string interfaceName = "TemplateData")
    {
        var fields = new Dictionary<string, InferredType>();

        collect_fields(nodes, fields);

        return generate_interface(fields, interfaceName);
    }


    /// <summary>
    ///     从符号表推导 TypeScript Data 接口
    /// </summary>
    /// <param name="symbolTable">编译期符号表。</param>
    /// <param name="interfaceName">接口名称。</param>
    /// <returns>TypeScript 接口源码。</returns>
    public string infer_from_symbol_table(SymbolTable symbolTable, string interfaceName = "TemplateData")
    {
        var fields = new Dictionary<string, InferredType>();

        foreach (var scope in symbolTable.all_scopes)
        foreach (var (name, isDeclared) in scope.references)
            if (!isDeclared && !fields.ContainsKey(name))
                fields[name] = new InferredType("any", false);

        return generate_interface(fields, interfaceName);
    }

    private void collect_fields(IReadOnlyList<DejaVuTemplateNode> nodes, Dictionary<string, InferredType> fields)
    {
        foreach (var node in nodes) collect_fields_from_node(node, fields);
    }

    private void collect_fields_from_node(DejaVuTemplateNode node, Dictionary<string, InferredType> fields)
    {
        switch (node)
        {
            case DejaVuCodeNode codeNode:
                collect_from_expression(codeNode.parsed_expression, fields);
                break;
            case DejaVuIfNode ifNode:
                collect_from_expression(ifNode.parsed_condition, fields);
                collect_fields(ifNode.children, fields);
                foreach (var elseIf in ifNode.else_if_nodes)
                {
                    collect_from_expression(elseIf.parsed_condition, fields);
                    collect_fields(elseIf.children, fields);
                }

                collect_fields(ifNode.else_children, fields);
                break;
            case DejaVuLoopNode loopNode:
                collect_from_expression(loopNode.parsed_expression, fields);
                if (loopNode.parsed_expression is IdentifierNode idNode)
                    merge_field(fields, idNode.name, new InferredType("any[]", true));
                else if (loopNode.parsed_expression is MemberAccessNode
                         {
                             @object: IdentifierNode memberIdNode
                         } memberNode)
                    merge_field(fields, memberIdNode.name,
                        new InferredType($"{{ {memberNode.member_name}: any[] }}", false));

                collect_fields(loopNode.children, fields);
                break;
            case DejaVuLetNode letNode:
                collect_from_expression(letNode.parsed_expression, fields);
                collect_fields(letNode.children, fields);
                break;
            case DejaVuWithNode withNode:
                collect_from_expression(withNode.parsed_expression, fields);
                collect_fields(withNode.children, fields);
                break;
            case DejaVuBlockNode blockNode:
                collect_fields(blockNode.children, fields);
                break;
            case DejaVuMatchNode matchNode:
                collect_from_expression(matchNode.parsed_expression, fields);
                collect_fields(matchNode.children, fields);
                break;
            case DejaVuRawNode rawNode:
                collect_fields(rawNode.children, fields);
                break;
        }
    }

    private void collect_from_expression(IExpressionNode? node, Dictionary<string, InferredType> fields)
    {
        if (node == null) return;

        switch (node)
        {
            case IdentifierNode id:
                merge_field(fields, id.name, new InferredType("any", false));
                break;
            case BinaryNode binary:
                collect_from_expression(binary.left, fields);
                collect_from_expression(binary.right, fields);
                break;
            case UnaryNode unary:
                collect_from_expression(unary.operand, fields);
                break;
            case MemberAccessNode member:
                collect_from_expression(member.@object, fields);
                if (member.@object is IdentifierNode parentId)
                    merge_field(fields, parentId.name, new InferredType($"{{ {member.member_name}: any }}", false));

                break;
            case CallNode call:
                collect_from_expression(call.function, fields);
                foreach (var arg in call.arguments) collect_from_expression(arg, fields);

                break;
            case IndexNode index:
                collect_from_expression(index.@object, fields);
                collect_from_expression(index.index, fields);
                break;
            case PipeNode pipe:
                collect_from_expression(pipe.left, fields);
                foreach (var arg in pipe.arguments) collect_from_expression(arg, fields);

                break;
        }
    }

    private static void merge_field(Dictionary<string, InferredType> fields, string name, InferredType type)
    {
        if (!fields.TryGetValue(name, out var existing))
        {
            fields[name] = type;
            return;
        }

        if (type.is_array && !existing.is_array) fields[name] = type;
    }

    private static string generate_interface(Dictionary<string, InferredType> fields, string interfaceName)
    {
        var sb = new StringBuilder();
        sb.AppendLine($"export interface {interfaceName} {{");

        foreach (var (name, type) in fields.OrderBy(f => f.Key)) sb.AppendLine($"    {name}: {type.type_name};");

        sb.AppendLine("}");
        return sb.ToString();
    }

    private sealed record InferredType(string type_name, bool is_array);
}