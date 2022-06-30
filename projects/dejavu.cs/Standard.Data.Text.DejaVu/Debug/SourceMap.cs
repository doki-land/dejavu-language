namespace Std.Data.Text.DejaVu.Debug;

/// <summary>
///     源码映射——编译后代码位置 → 模板源码位置
/// </summary>
public sealed class SourceMap
{
    private readonly List<SourceMapping> _mappings = [];


    /// <summary>
    ///     映射条目
    /// </summary>
    public IReadOnlyList<SourceMapping> mappings => _mappings;


    /// <summary>
    ///     添加映射
    /// </summary>
    public void add_mapping(int generatedLine, int generatedColumn, int sourceLine, int sourceColumn,
        string sourceFile = "")
    {
        _mappings.Add(new SourceMapping
        {
            generated_line = generatedLine,
            generated_column = generatedColumn,
            source_line = sourceLine,
            source_column = sourceColumn,
            source_file = sourceFile
        });
    }


    /// <summary>
    ///     从生成代码位置查找源码位置
    /// </summary>
    public SourceMapping? find_source_position(int generatedLine, int generatedColumn)
    {
        SourceMapping? best = null;

        foreach (var mapping in _mappings)
            if (mapping.generated_line < generatedLine ||
                (mapping.generated_line == generatedLine && mapping.generated_column <= generatedColumn))
                if (best == null ||
                    mapping.generated_line > best.generated_line ||
                    (mapping.generated_line == best.generated_line && mapping.generated_column > best.generated_column))
                    best = mapping;

        return best;
    }


    /// <summary>
    ///     从模板 AST 构建源码映射
    /// </summary>
    public static SourceMap build_from_nodes(IReadOnlyList<DejaVuTemplateNode> nodes, string sourceFile = "")
    {
        var sourceMap = new SourceMap();
        var generatedLine = 1;

        foreach (var node in nodes) build_mapping_for_node(sourceMap, node, ref generatedLine, sourceFile);

        return sourceMap;
    }

    private static void build_mapping_for_node(SourceMap sourceMap, DejaVuTemplateNode node, ref int generatedLine,
        string sourceFile)
    {
        var sourceLine = node.source_line > 0 ? node.source_line : 0;
        var sourceColumn = node.source_column > 0 ? node.source_column : 0;

        sourceMap.add_mapping(generatedLine, 0, sourceLine, sourceColumn, sourceFile);

        switch (node)
        {
            case DejaVuIfNode ifNode:
                generatedLine++;
                foreach (var child in ifNode.children)
                    build_mapping_for_node(sourceMap, child, ref generatedLine, sourceFile);

                generatedLine++;
                break;
            case DejaVuLoopNode loopNode:
                generatedLine++;
                foreach (var child in loopNode.children)
                    build_mapping_for_node(sourceMap, child, ref generatedLine, sourceFile);

                generatedLine++;
                break;
            case DejaVuBlockNode blockNode:
                foreach (var child in blockNode.children)
                    build_mapping_for_node(sourceMap, child, ref generatedLine, sourceFile);

                break;
            case DejaVuRawNode rawNode:
                foreach (var child in rawNode.children)
                    build_mapping_for_node(sourceMap, child, ref generatedLine, sourceFile);

                break;
        }

        generatedLine++;
    }
}