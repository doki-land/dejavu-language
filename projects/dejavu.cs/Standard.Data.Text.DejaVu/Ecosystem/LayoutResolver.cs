namespace Std.Data.Text.DejaVu.Ecosystem;

/// <summary>
///     布局解析器——解析模板继承链、嵌套布局、Content Placeholder。
///     支持多层 extends 嵌套和 block 覆盖传播。
/// </summary>
public sealed class LayoutResolver
{
    /// <summary>
    ///     解析布局继承链——返回从根布局到子模板的完整继承路径
    /// </summary>
    /// <param name="templatePath">起始模板路径。</param>
    /// <param name="templateLoader">模板加载器。</param>
    /// <returns>布局继承链（索引 0 为根布局，最后一个为当前模板）。</returns>
    public LayoutChain resolve_chain(string templatePath, ITemplateLoader templateLoader)
    {
        var chain = new List<LayoutNode>();
        var visited = new HashSet<string>();
        var currentPath = templatePath;

        while (!string.IsNullOrEmpty(currentPath))
        {
            if (!visited.Add(currentPath))
                return new LayoutChain(chain, LayoutResolveStatus.circular_inheritance, $"检测到循环继承: {currentPath}");

            var source = templateLoader.load(currentPath);
            if (source == null)
                return new LayoutChain(chain, LayoutResolveStatus.template_not_found, $"模板未找到: {currentPath}");

            var parser = new DejaVuParser("doki");
            var parseResult = parser.parse(source);

            var extendsNode = parseResult.nodes.OfType<DejaVuExtendsNode>().FirstOrDefault();
            var blocks = collect_blocks(parseResult.nodes);
            var contentPlaceholders = collect_content_placeholders(parseResult.nodes);

            chain.Add(new LayoutNode
            {
                template_path = currentPath,
                source = source,
                blocks = blocks,
                content_placeholders = contentPlaceholders,
                parent_template_path = extendsNode?.parent_template.Trim('\'', '"')
            });

            currentPath = extendsNode?.parent_template.Trim('\'', '"') ?? string.Empty;
            if (!string.IsNullOrEmpty(currentPath) && !currentPath.Contains('.') && !currentPath.Contains('/') &&
                !currentPath.Contains('\\')) currentPath = templateLoader.resolve_path(currentPath);
        }

        chain.Reverse();

        return new LayoutChain(chain, LayoutResolveStatus.success, null);
    }


    /// <summary>
    ///     合并布局链——将子模板的 block 覆盖传播到根布局
    /// </summary>
    /// <param name="chain">布局继承链。</param>
    /// <returns>合并后的 block 映射（block 名 → 最终内容节点列表）。</returns>
    public Dictionary<string, MergedBlock> merge_blocks(LayoutChain chain)
    {
        var merged = new Dictionary<string, MergedBlock>();

        foreach (var node in chain.nodes)
        foreach (var (name, blockInfo) in node.blocks)
            if (!merged.TryGetValue(name, out var existing))
            {
                merged[name] = new MergedBlock
                {
                    name = name,
                    default_content = blockInfo.children,
                    override_content = blockInfo.children,
                    defined_in = node.template_path,
                    override_from = node.template_path
                };
            }
            else
            {
                var hasSuper = blockInfo.children.Any(c => c is DejaVuSuperNode);
                merged[name] = new MergedBlock
                {
                    name = name,
                    default_content = existing.default_content,
                    override_content = hasSuper
                        ? merge_with_super(existing.override_content, blockInfo.children)
                        : blockInfo.children,
                    defined_in = existing.defined_in,
                    override_from = node.template_path
                };
            }

        return merged;
    }


    /// <summary>
    ///     渲染布局——从根布局开始，逐层应用 block 覆盖
    /// </summary>
    /// <param name="chain">布局继承链。</param>
    /// <param name="mergedBlocks">合并后的 block 映射。</param>
    /// <returns>渲染后的完整模板节点列表。</returns>
    public List<DejaVuTemplateNode> render_layout(LayoutChain chain, Dictionary<string, MergedBlock> mergedBlocks)
    {
        if (chain.nodes.Count == 0) return [];

        var rootNode = chain.nodes[0];
        return replace_blocks(rootNode.source, rootNode, mergedBlocks);
    }

    private List<DejaVuTemplateNode> replace_blocks(string source, LayoutNode layoutNode,
        Dictionary<string, MergedBlock> mergedBlocks)
    {
        var parser = new DejaVuParser("doki");
        var parseResult = parser.parse(source);
        var result = new List<DejaVuTemplateNode>();

        foreach (var node in parseResult.nodes)
            if (node is DejaVuBlockNode blockNode)
            {
                if (mergedBlocks.TryGetValue(blockNode.name, out var merged))
                    result.AddRange(merged.override_content);
                else
                    result.AddRange(blockNode.children);
            }
            else if (node is DejaVuExtendsNode)
            {
            }
            else
            {
                result.Add(node);
            }

        return result;
    }

    private static List<DejaVuTemplateNode> merge_with_super(List<DejaVuTemplateNode> defaultContent,
        List<DejaVuTemplateNode> overrideContent)
    {
        var result = new List<DejaVuTemplateNode>();

        foreach (var node in overrideContent)
            if (node is DejaVuSuperNode)
                result.AddRange(defaultContent);
            else
                result.Add(node);

        return result;
    }

    private static Dictionary<string, BlockInfo> collect_blocks(IReadOnlyList<DejaVuTemplateNode> nodes)
    {
        var blocks = new Dictionary<string, BlockInfo>();

        foreach (var node in nodes)
            if (node is DejaVuBlockNode blockNode)
                blocks[blockNode.name] = new BlockInfo
                {
                    name = blockNode.name,
                    children = [.. blockNode.children]
                };

        return blocks;
    }

    private static List<ContentPlaceholder> collect_content_placeholders(IReadOnlyList<DejaVuTemplateNode> nodes)
    {
        var placeholders = new List<ContentPlaceholder>();

        foreach (var node in nodes)
            if (node is DejaVuBlockNode blockNode)
                placeholders.Add(new ContentPlaceholder
                {
                    name = blockNode.name,
                    has_default_content = blockNode.children.Count > 0
                });

        return placeholders;
    }
}