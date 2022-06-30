namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     组件注册表——可复用的模板组件（类似 ViewComponent / Partial with parameters）。
///     支持 slot 插槽和 props 参数传递。
/// </summary>
public sealed class ComponentRegistry
{
    private readonly Dictionary<string, ComponentDefinition> _components = new();


    /// <summary>
    ///     所有已注册组件
    /// </summary>
    public IReadOnlyDictionary<string, ComponentDefinition> components => _components;


    /// <summary>
    ///     注册组件
    /// </summary>
    /// <param name="name">组件名。</param>
    /// <param name="templateSource">组件模板源码。</param>
    /// <param name="props">组件参数定义。</param>
    /// <param name="slots">组件插槽定义。</param>
    public void register(string name, string templateSource, List<ComponentProp>? props = null,
        List<ComponentSlot>? slots = null)
    {
        var parser = new DejaVuParser("doki");
        var parseResult = parser.parse(templateSource);

        _components[name] = new ComponentDefinition
        {
            name = name,
            template_source = templateSource,
            nodes = [.. parseResult.nodes],
            props = props ?? [],
            slots = slots ?? []
        };
    }


    /// <summary>
    ///     注册组件（从文件加载）
    /// </summary>
    public void register_from_file(string name, string filePath, List<ComponentProp>? props = null,
        List<ComponentSlot>? slots = null)
    {
        if (!File.Exists(filePath)) return;

        var source = File.ReadAllText(filePath);
        register(name, source, props, slots);
    }


    /// <summary>
    ///     检查组件是否已注册
    /// </summary>
    public bool has_component(string name)
    {
        return _components.ContainsKey(name);
    }


    /// <summary>
    ///     获取组件定义
    /// </summary>
    public ComponentDefinition? get_component(string name)
    {
        return _components.GetValueOrDefault(name);
    }


    /// <summary>
    ///     渲染组件——将参数和插槽内容注入组件模板
    /// </summary>
    /// <param name="name">组件名。</param>
    /// <param name="props">传入参数。</param>
    /// <param name="slots">传入插槽内容。</param>
    /// <returns>渲染后的节点列表。</returns>
    public List<DejaVuTemplateNode> render_component(string name, Dictionary<string, object> props,
        Dictionary<string, List<DejaVuTemplateNode>> slots)
    {
        if (!_components.TryGetValue(name, out var component))
            return [new DejaVuTextNode { text = $"<!-- 组件 \"{name}\" 未注册 -->" }];

        var result = new List<DejaVuTemplateNode>();

        foreach (var node in component.nodes) result.AddRange(expand_node(node, props, slots, component));

        return result;
    }

    private List<DejaVuTemplateNode> expand_node(DejaVuTemplateNode node, Dictionary<string, object> props,
        Dictionary<string, List<DejaVuTemplateNode>> slots, ComponentDefinition component)
    {
        switch (node)
        {
            case DejaVuCodeNode codeNode:
                var expandedCode = expand_expressions(codeNode.code, props);
                return
                [
                    new DejaVuCodeNode
                    {
                        code = expandedCode,
                        parsed_expression = codeNode.parsed_expression
                    }
                ];

            case DejaVuIfNode ifNode:
                var ifChildren = new List<DejaVuTemplateNode>();
                foreach (var child in ifNode.children) ifChildren.AddRange(expand_node(child, props, slots, component));

                var elseIfNodes = new List<DejaVuElseIfNode>();
                foreach (var elseIf in ifNode.else_if_nodes)
                {
                    var elseIfChildren = new List<DejaVuTemplateNode>();
                    foreach (var child in elseIf.children)
                        elseIfChildren.AddRange(expand_node(child, props, slots, component));

                    elseIfNodes.Add(new DejaVuElseIfNode
                    {
                        condition = elseIf.condition,
                        parsed_condition = elseIf.parsed_condition,
                        children = elseIfChildren
                    });
                }

                var elseChildren = new List<DejaVuTemplateNode>();
                foreach (var child in ifNode.else_children)
                    elseChildren.AddRange(expand_node(child, props, slots, component));

                return
                [
                    new DejaVuIfNode
                    {
                        condition = ifNode.condition,
                        parsed_condition = ifNode.parsed_condition,
                        children = ifChildren,
                        else_if_nodes = elseIfNodes,
                        else_children = elseChildren
                    }
                ];

            case DejaVuLoopNode loopNode:
                var loopChildren = new List<DejaVuTemplateNode>();
                foreach (var child in loopNode.children)
                    loopChildren.AddRange(expand_node(child, props, slots, component));

                return
                [
                    new DejaVuLoopNode
                    {
                        expression = loopNode.expression,
                        parsed_expression = loopNode.parsed_expression,
                        item_name = loopNode.item_name,
                        children = loopChildren
                    }
                ];

            case DejaVuBlockNode blockNode:
                var slotName = blockNode.name;
                if (slots.TryGetValue(slotName, out var slotContent)) return slotContent;

                var blockChildren = new List<DejaVuTemplateNode>();
                foreach (var child in blockNode.children)
                    blockChildren.AddRange(expand_node(child, props, slots, component));

                return
                [
                    new DejaVuBlockNode
                    {
                        name = blockNode.name,
                        children = blockChildren
                    }
                ];

            default:
                return [node];
        }
    }

    private static string expand_expressions(string code, Dictionary<string, object> props)
    {
        foreach (var (key, value) in props) code = code.Replace($"{{{{{key}}}}}", value?.ToString() ?? "");

        return code;
    }
}