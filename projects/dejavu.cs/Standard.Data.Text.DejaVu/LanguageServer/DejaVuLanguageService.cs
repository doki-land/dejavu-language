using Std.Data.Text.DejaVu.CodeGen;
using Std.Data.Text.DejaVu.Filters;
using Std.Data.Text.DejaVu.Optimizer;
using Std.Data.Text.Diagnostics;

namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     DejaVu 语言服务——提供文档诊断、自动补全、悬停提示等 LSP 功能。
///     作为 LSP 服务器后端，不依赖具体的 LSP 传输协议。
/// </summary>
public sealed class DejaVuLanguageService
{
    private readonly DejaVuCompiler _compiler;
    private readonly Dictionary<string, DocumentState> _documents = new();
    private readonly FilterRegistry _filters;
    private readonly TemplateStandardLibrary _standard_library;


    /// <summary>
    ///     创建 DejaVu 语言服务
    /// </summary>
    public DejaVuLanguageService()
    {
        _compiler = new DejaVuCompiler(new DejaVuParser("doki"));
        _filters = new FilterRegistry();
        _standard_library = new TemplateStandardLibrary();
    }


    /// <summary>
    ///     打开文档——解析并缓存
    /// </summary>
    /// <param name="uri">文档 URI。</param>
    /// <param name="source">文档源码。</param>
    /// <returns>初始诊断。</returns>
    public List<LspDiagnostic> open_document(string uri, string source)
    {
        var state = analyze_document(uri, source);
        _documents[uri] = state;
        return state.diagnostics;
    }


    /// <summary>
    ///     更新文档——增量解析
    /// </summary>
    /// <param name="uri">文档 URI。</param>
    /// <param name="source">更新后的源码。</param>
    /// <returns>更新后的诊断。</returns>
    public List<LspDiagnostic> update_document(string uri, string source)
    {
        var state = analyze_document(uri, source);
        _documents[uri] = state;
        return state.diagnostics;
    }


    /// <summary>
    ///     关闭文档——移除缓存
    /// </summary>
    public void close_document(string uri)
    {
        _documents.Remove(uri);
    }


    /// <summary>
    ///     获取自动补全列表
    /// </summary>
    /// <param name="uri">文档 URI。</param>
    /// <param name="line">行号（0-based）。</param>
    /// <param name="character">列号（0-based）。</param>
    /// <returns>补全项列表。</returns>
    public List<LspCompletionItem> get_completions(string uri, int line, int character)
    {
        var items = new List<LspCompletionItem>();

        if (!_documents.TryGetValue(uri, out var state)) return items;

        add_filter_completions(items);
        add_standard_helper_completions(items);
        add_keyword_completions(items);
        add_variable_completions(items, state);

        return items;
    }


    /// <summary>
    ///     获取悬停提示
    /// </summary>
    /// <param name="uri">文档 URI。</param>
    /// <param name="line">行号（0-based）。</param>
    /// <param name="character">列号（0-based）。</param>
    /// <returns>悬停信息，或 null。</returns>
    public LspHover? get_hover(string uri, int line, int character)
    {
        if (!_documents.TryGetValue(uri, out var state)) return null;

        var word = extract_word_at_position(state.source, line, character);
        if (string.IsNullOrEmpty(word)) return null;

        var hoverContent = resolve_hover_content(word, state);
        if (hoverContent == null) return null;

        return new LspHover
        {
            range = new LspRange
            {
                start = new LspPosition { line = line, character = character },
                end = new LspPosition { line = line, character = character + word.Length }
            },
            contents = hoverContent
        };
    }


    /// <summary>
    ///     获取文档符号列表（大纲视图）
    /// </summary>
    public List<DocumentSymbol> get_document_symbols(string uri)
    {
        if (!_documents.TryGetValue(uri, out var state)) return [];

        if (state.symbol_table == null) return [];

        var symbols = new List<DocumentSymbol>();

        foreach (var blockName in state.symbol_table.blocks)
            symbols.Add(new DocumentSymbol
            {
                name = blockName,
                kind = SymbolKind.method,
                detail = "block"
            });

        if (state.symbol_table.parent_template != null)
            symbols.Add(new DocumentSymbol
            {
                name = $"extends: {state.symbol_table.parent_template}",
                kind = SymbolKind.module,
                detail = "继承"
            });

        foreach (var includePath in state.symbol_table.included_templates)
            symbols.Add(new DocumentSymbol
            {
                name = $"include: {includePath}",
                kind = SymbolKind.file,
                detail = "引入"
            });

        return symbols;
    }

    private DocumentState analyze_document(string uri, string source)
    {
        var diagnostics = new DiagnosticSink();

        var parser = new DejaVuParser("doki", diagnostics);
        var parseResult = parser.parse(source);

        var optimizer = new TemplateOptimizer();
        var optimizedNodes = optimizer.optimize([.. parseResult.nodes]);

        var symbolResolver = new SymbolResolver(diagnostics);
        var symbolTable = symbolResolver.resolve(optimizedNodes);

        var typeChecker = new TypeChecker(diagnostics, _filters);
        var inferredTypes = typeChecker.check(optimizedNodes);

        var lspDiagnostics = LspDiagnosticConverter.convert(diagnostics);

        return new DocumentState
        {
            source = source,
            nodes = optimizedNodes,
            symbol_table = symbolTable,
            inferred_types = inferredTypes,
            diagnostics = lspDiagnostics
        };
    }

    private void add_filter_completions(List<LspCompletionItem> items)
    {
        var filterNames = new[]
        {
            "uppercase", "lowercase", "trim", "length", "reverse",
            "abs", "round", "floor", "ceil",
            "first", "last", "count", "join",
            "date", "datetime",
            "default", "escape", "safe"
        };

        foreach (var name in filterNames)
            items.Add(new LspCompletionItem
            {
                label = name,
                kind = 3,
                detail = $"过滤器: {name}",
                documentation = $"DejaVu 内置过滤器 {name}",
                insert_text = name
            });
    }

    private void add_standard_helper_completions(List<LspCompletionItem> items)
    {
        foreach (var (name, helper) in _standard_library.helpers)
        {
            var paramList = string.Join(", ", helper.parameters.Select(p => p.name));
            items.Add(new LspCompletionItem
            {
                label = name,
                kind = 3,
                detail = $"{helper.description} ({paramList})",
                documentation = helper.description,
                insert_text = $"{name}({paramList})"
            });
        }
    }

    private void add_keyword_completions(List<LspCompletionItem> items)
    {
        var keywords = new[]
        {
            ("if", "条件判断", 17),
            ("else if", "条件分支", 17),
            ("else", "默认分支", 17),
            ("loop", "循环遍历", 17),
            ("loop in", "带变量循环", 17),
            ("let", "局部变量绑定", 17),
            ("with", "作用域别名", 17),
            ("block", "块定义", 17),
            ("extends", "模板继承", 17),
            ("include", "引入子模板", 17),
            ("raw", "原始输出", 17),
            ("end", "结束标签", 17),
            ("super()", "父模板默认内容", 3)
        };

        foreach (var (keyword, description, kind) in keywords)
            items.Add(new LspCompletionItem
            {
                label = keyword,
                kind = kind,
                detail = description,
                documentation = description,
                insert_text = keyword
            });
    }

    private void add_variable_completions(List<LspCompletionItem> items, DocumentState state)
    {
        foreach (var (name, type) in state.inferred_types)
            items.Add(new LspCompletionItem
            {
                label = name,
                kind = 6,
                detail = $"变量: {type}",
                documentation = $"模板变量 \"{name}\"，类型: {type}",
                insert_text = name
            });
    }

    private string? resolve_hover_content(string word, DocumentState state)
    {
        if (_standard_library.has_helper(word))
        {
            var helper = _standard_library.get_helper(word)!;
            var paramList = string.Join(", ", helper.parameters.Select(p => $"{p.name}: {p.type}"));
            return $"**{word}**({paramList}): {helper.description}\n\n输出类型: {helper.output_type}";
        }

        if (_filters.has_filter(word)) return $"**{word}** — DejaVu 内置过滤器";

        if (state.inferred_types.TryGetValue(word, out var type)) return $"**{word}**: `{type}` — 模板变量";

        if (state.symbol_table?.blocks.Contains(word) == true) return $"**block {word}** — 模板块定义";

        return null;
    }

    private static string extract_word_at_position(string source, int line, int character)
    {
        var lines = source.Split('\n');
        if (line < 0 || line >= lines.Length) return string.Empty;

        var lineText = lines[line];
        if (character < 0 || character >= lineText.Length) return string.Empty;

        var start = character;
        while (start > 0 && is_word_char(lineText[start - 1])) start--;

        var end = character;
        while (end < lineText.Length && is_word_char(lineText[end])) end++;

        return start < end ? lineText[start..end] : string.Empty;
    }

    private static bool is_word_char(char ch)
    {
        return char.IsLetterOrDigit(ch) || ch == '_';
    }
}