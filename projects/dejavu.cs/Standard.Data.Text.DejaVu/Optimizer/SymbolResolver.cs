using Std.Data.Text.DejaVu.Expressions;
using Std.Data.Text.Diagnostics;

namespace Std.Data.Text.DejaVu.Optimizer;

/// <summary>
///     编译期符号解析器——变量作用域分析、引用验证、继承链检查。
/// </summary>
public sealed class SymbolResolver
{
    private readonly DiagnosticSink _diagnostics;


    /// <summary>
    ///     创建符号解析器
    /// </summary>
    /// <param name="diagnostics">诊断消息收集器。</param>
    public SymbolResolver(DiagnosticSink diagnostics)
    {
        _diagnostics = diagnostics;
    }


    /// <summary>
    ///     对编译后的模板节点执行符号解析
    /// </summary>
    /// <returns>解析出的符号表。</returns>
    public SymbolTable resolve(IReadOnlyList<DejaVuTemplateNode> nodes)
    {
        var globalScope = new Scope(null, "<global>");
        var symbolTable = new SymbolTable(globalScope);

        resolve_nodes(nodes, globalScope, symbolTable);

        validate_scopes(symbolTable);

        return symbolTable;
    }

    private void resolve_nodes(IReadOnlyList<DejaVuTemplateNode> nodes, Scope parentScope, SymbolTable symbolTable)
    {
        foreach (var node in nodes) resolve_node(node, parentScope, symbolTable);
    }

    private void resolve_node(DejaVuTemplateNode node, Scope parentScope, SymbolTable symbolTable)
    {
        switch (node)
        {
            case DejaVuIfNode ifNode:
                resolve_if_node(ifNode, parentScope, symbolTable);
                break;
            case DejaVuLoopNode loopNode:
                resolve_loop_node(loopNode, parentScope, symbolTable);
                break;
            case DejaVuLetNode letNode:
                resolve_let_node(letNode, parentScope, symbolTable);
                break;
            case DejaVuWithNode withNode:
                resolve_with_node(withNode, parentScope, symbolTable);
                break;
            case DejaVuBlockNode blockNode:
                resolve_block_node(blockNode, parentScope, symbolTable);
                break;
            case DejaVuCodeNode codeNode:
                resolve_expression_references(codeNode.parsed_expression, parentScope);
                break;
            case DejaVuExtendsNode extendsNode:
                resolve_extends_node(extendsNode, symbolTable);
                break;
            case DejaVuIncludeNode includeNode:
                resolve_include_node(includeNode, symbolTable);
                break;
            case DejaVuMatchNode matchNode:
                resolve_match_node(matchNode, parentScope, symbolTable);
                break;
            case DejaVuRawNode rawNode:
                resolve_nodes(rawNode.children, parentScope, symbolTable);
                break;
        }
    }

    private void resolve_if_node(DejaVuIfNode ifNode, Scope parentScope, SymbolTable symbolTable)
    {
        resolve_expression_references(ifNode.parsed_condition, parentScope);
        resolve_nodes(ifNode.children, parentScope, symbolTable);

        foreach (var elseIfNode in ifNode.else_if_nodes)
        {
            resolve_expression_references(elseIfNode.parsed_condition, parentScope);
            resolve_nodes(elseIfNode.children, parentScope, symbolTable);
        }

        resolve_nodes(ifNode.else_children, parentScope, symbolTable);
    }

    private void resolve_loop_node(DejaVuLoopNode loopNode, Scope parentScope, SymbolTable symbolTable)
    {
        resolve_expression_references(loopNode.parsed_expression, parentScope);

        var loopScope = new Scope(parentScope, "loop");
        var itemName = loopNode.item_name ?? "item";
        loopScope.declare(itemName, SymbolKind.iteration_variable);
        loopScope.declare("index", SymbolKind.iteration_variable);

        symbolTable.add_scope(loopScope);
        resolve_nodes(loopNode.children, loopScope, symbolTable);
    }

    private void resolve_let_node(DejaVuLetNode letNode, Scope parentScope, SymbolTable symbolTable)
    {
        resolve_expression_references(letNode.parsed_expression, parentScope);

        var letScope = new Scope(parentScope, $"let:{letNode.variable_name}");
        letScope.declare(letNode.variable_name, SymbolKind.local_variable);

        symbolTable.add_scope(letScope);
        resolve_nodes(letNode.children, letScope, symbolTable);
    }

    private void resolve_with_node(DejaVuWithNode withNode, Scope parentScope, SymbolTable symbolTable)
    {
        resolve_expression_references(withNode.parsed_expression, parentScope);

        var withScope = new Scope(parentScope, $"with:{withNode.alias_name}");

        // with 块内的 .member 访问现在从别名对象解析，声明别名
        if (!string.IsNullOrEmpty(withNode.alias_name)) withScope.declare(withNode.alias_name, SymbolKind.scope_alias);

        symbolTable.add_scope(withScope);
        resolve_nodes(withNode.children, withScope, symbolTable);
    }

    private void resolve_block_node(DejaVuBlockNode blockNode, Scope parentScope, SymbolTable symbolTable)
    {
        var blockScope = new Scope(parentScope, $"block:{blockNode.name}");
        symbolTable.add_scope(blockScope);
        symbolTable.register_block(blockNode.name);
        resolve_nodes(blockNode.children, blockScope, symbolTable);
    }

    private void resolve_extends_node(DejaVuExtendsNode extendsNode, SymbolTable symbolTable)
    {
        var parentTemplate = extendsNode.parent_template.Trim('\'', '"');
        symbolTable.parent_template = parentTemplate;
    }

    private void resolve_include_node(DejaVuIncludeNode includeNode, SymbolTable symbolTable)
    {
        var templatePath = includeNode.template_path.Trim('\'', '"');
        symbolTable.included_templates.Add(templatePath);
    }

    private void resolve_match_node(DejaVuMatchNode matchNode, Scope parentScope, SymbolTable symbolTable)
    {
        resolve_expression_references(matchNode.parsed_expression, parentScope);
        resolve_nodes(matchNode.children, parentScope, symbolTable);
    }


    /// <summary>
    ///     从表达式 AST 中收集标识符引用
    /// </summary>
    private void resolve_expression_references(IExpressionNode? node, Scope scope)
    {
        if (node == null) return;

        switch (node)
        {
            case IdentifierNode identifier:
                scope.reference(identifier.name);
                break;
            case BinaryNode binary:
                resolve_expression_references(binary.left, scope);
                resolve_expression_references(binary.right, scope);
                break;
            case UnaryNode unary:
                resolve_expression_references(unary.operand, scope);
                break;
            case MemberAccessNode memberAccess:
                resolve_expression_references(memberAccess.@object, scope);
                break;
            case CallNode call:
                resolve_expression_references(call.function, scope);
                foreach (var arg in call.arguments) resolve_expression_references(arg, scope);

                break;
            case IndexNode index:
                resolve_expression_references(index.@object, scope);
                resolve_expression_references(index.index, scope);
                break;
            case PipeNode pipe:
                resolve_expression_references(pipe.left, scope);
                foreach (var arg in pipe.arguments) resolve_expression_references(arg, scope);

                break;
        }
    }


    /// <summary>
    ///     验证所有作用域中的引用是否合法
    /// </summary>
    private void validate_scopes(SymbolTable symbolTable)
    {
        foreach (var scope in symbolTable.all_scopes)
        foreach (var (name, isDeclared) in scope.references)
        {
            if (isDeclared) continue;

            // 检查是否是全局上下文变量（运行期绑定）
            if (symbolTable.global_scope.is_declared(name))
            {
                scope.mark_declared(name);
                continue;
            }

            // 未声明的变量——在编译期报告警告（运行期可能从上下文中绑定）
            _diagnostics.report_warning(
                string.Empty,
                default,
                "UndefinedVariable",
                $"未声明的变量 \"{name}\" 在作用域 \"{scope.name}\" 中被引用，运行期将从模板上下文中解析"
            );
        }
    }
}