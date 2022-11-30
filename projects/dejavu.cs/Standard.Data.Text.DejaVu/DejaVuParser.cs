using System.Text;
using Std.Data.Text.DejaVu.Expressions;
using Std.Data.Text.Diagnostics;
using Std.Data.Text.Parsing;
using OakTextReader = Std.Data.Text.Syntax.TextReader;

namespace Std.Data.Text.DejaVu;

/// <summary>
///     DejaVu 模板解析器
/// </summary>
public sealed class DejaVuParser
{
    /// <summary>
    ///     获取解析过程中的诊断信息。
    /// </summary>
    public DiagnosticSink get_diagnostics()
    {
        return _diagnostics;
    }

    #region 字段

    private readonly DiagnosticSink _diagnostics;
    private readonly DejaVuLanguage _language;
    private readonly ExpressionParser? _expression_parser;

    private static readonly HashSet<string> _block_keywords =
    [
        new(StringComparer.Ordinal),
        "if", "loop", "match", "block"
    ];

    #endregion

    #region 构造函数

    /// <summary>
    ///     创建 DejaVu 解析器
    /// </summary>
    /// <param name="language">模板语言。</param>
    /// <param name="diagnostics">诊断消息收集器。</param>
    /// <param name="expressionParser">表达式解析器（若提供则预解析表达式）。</param>
    public DejaVuParser(DejaVuLanguage language, DiagnosticSink? diagnostics = null,
        ExpressionParser? expressionParser = null)
    {
        _language = language;
        _diagnostics = diagnostics ?? new DiagnosticSink();
        _expression_parser = expressionParser;
    }


    /// <summary>
    ///     创建 DejaVu 解析器
    /// </summary>
    /// <param name="languageName">模板语言名称，可选值："dora" 或 "doki"。</param>
    /// <param name="diagnostics">诊断消息收集器。</param>
    /// <param name="expressionParser">表达式解析器（若提供则预解析表达式）。</param>
    public DejaVuParser(string languageName, DiagnosticSink? diagnostics = null,
        ExpressionParser? expressionParser = null)
    {
        try
        {
            _language = DejaVuLanguage.get_by_name(languageName);
        }
        catch (ArgumentException ex)
        {
            _language = DejaVuLanguage.dora;
            _diagnostics = diagnostics ?? new DiagnosticSink();
            _expression_parser = expressionParser;
            _diagnostics.report_error("", default, "InvalidTemplateType", ex.Message);
            return;
        }

        _diagnostics = diagnostics ?? new DiagnosticSink();
        _expression_parser = expressionParser;
    }

    #endregion

    #region 公共方法

    /// <summary>
    ///     解析 DejaVu 模板
    /// </summary>
    /// <returns>解析结果。</returns>
    public DejaVuParseResult parse(string source)
    {
        var nodes = new List<DejaVuTemplateNode>();
        var reader = new OakTextReader(source);

        parse_template(reader, nodes);

        return new DejaVuParseResult
        {
            nodes = nodes,
            template_type = _language.name
        };
    }


    /// <summary>
    ///     编译模板——解析 + 优化，返回可缓存的编译产物
    /// </summary>
    public CompiledTemplate compile(string source, string templatePath = "")
    {
        var parseResult = parse(source);
        return CompiledTemplate.compile(parseResult, templatePath, DateTimeOffset.UtcNow);
    }

    #endregion

    #region 解析核心

    /// <summary>
    ///     解析模板内容
    /// </summary>
    private void parse_template(OakTextReader reader, List<DejaVuTemplateNode> nodes)
    {
        var sb = new StringBuilder();

        while (!reader.is_at_end)
        {
            var text = read_until_delimiter(reader, out var isCode, out var isComment);

            if (!string.IsNullOrEmpty(text)) sb.Append(text);

            if (reader.is_at_end) break;

            if (isComment)
            {
                skip_comment(reader);
            }
            else if (isCode)
            {
                if (sb.Length > 0)
                {
                    nodes.Add(new DejaVuTextNode { text = sb.ToString() });
                    sb.Clear();
                }

                process_code_block(reader, nodes);
            }
        }

        if (sb.Length > 0) nodes.Add(new DejaVuTextNode { text = sb.ToString() });
    }


    /// <summary>
    ///     读取文本直到遇到分隔符
    /// </summary>
    private string read_until_delimiter(OakTextReader reader, out bool isCode, out bool isComment)
    {
        isCode = false;
        isComment = false;

        var start = reader.position;

        while (!reader.is_at_end)
        {
            if (reader.peek() == _language.comment_start[0])
                if (check_delimiter(reader, _language.comment_start))
                {
                    isComment = true;
                    break;
                }

            if (reader.peek() == _language.opening_delimiter[0])
                if (check_delimiter(reader, _language.opening_delimiter))
                    if (!_language.comment_start.StartsWith(_language.opening_delimiter) ||
                        !check_delimiter(reader, _language.comment_start))
                    {
                        isCode = true;
                        break;
                    }

            reader.advance();
        }

        return reader.slice(start, reader.position - start);
    }


    /// <summary>
    ///     检查当前位置是否匹配指定分隔符
    /// </summary>
    private bool check_delimiter(OakTextReader reader, string delimiter)
    {
        for (var i = 0; i < delimiter.Length; i++)
            if (reader.peek(i) != delimiter[i])
                return false;

        return true;
    }


    /// <summary>
    ///     跳过注释
    /// </summary>
    private void skip_comment(OakTextReader reader)
    {
        for (var i = 0; i < _language.comment_start.Length; i++) reader.advance();

        while (!reader.is_at_end)
        {
            if (reader.peek() == _language.comment_end[0])
                if (check_delimiter(reader, _language.comment_end))
                {
                    for (var i = 0; i < _language.comment_end.Length; i++) reader.advance();

                    return;
                }

            reader.advance();
        }

        _diagnostics.report_error("", default, "UnclosedComment", "未闭合的注释。");
    }

    #endregion

    #region 代码块处理

    /// <summary>
    ///     读取代码块内容
    /// </summary>
    private string read_code_content(OakTextReader reader)
    {
        for (var i = 0; i < _language.opening_delimiter.Length; i++) reader.advance();

        var codeStart = reader.position;

        while (!reader.is_at_end)
        {
            if (reader.peek() == _language.closing_delimiter[0])
                if (check_delimiter(reader, _language.closing_delimiter))
                    break;

            reader.advance();
        }

        var codeContent = reader.slice(codeStart, reader.position - codeStart).Trim();

        if (reader.is_at_end)
        {
            _diagnostics.report_error("", default, "UnclosedCodeBlock", "未闭合的代码块。");
            return codeContent;
        }

        for (var i = 0; i < _language.closing_delimiter.Length; i++) reader.advance();

        return codeContent;
    }


    /// <summary>
    ///     检查 end 指令类型
    /// </summary>
    /// <param name="codeContent">代码块内容。</param>
    /// <param name="expectedType">期望的块类型。</param>
    /// <param name="actualType">实际的块类型（仅显式 end 时有效）。</param>
    /// <returns>end 检查结果。</returns>
    private static EndCheckResult check_end_directive(string codeContent, string expectedType, out string? actualType)
    {
        actualType = null;

        if (codeContent == "end") return EndCheckResult.end_stack;

        if (codeContent.StartsWith("end "))
        {
            var type = codeContent["end ".Length..].Trim();
            actualType = type;
            return EndCheckResult.end_explicit;
        }

        return EndCheckResult.not_end;
    }


    /// <summary>
    ///     处理 end 闭合，返回是否匹配成功
    /// </summary>
    private bool handle_end(string codeContent, string expectedType)
    {
        var result = check_end_directive(codeContent, expectedType, out var actualType);

        switch (result)
        {
            case EndCheckResult.end_stack:
                return true;

            case EndCheckResult.end_explicit:
                if (actualType != expectedType)
                    _diagnostics.report_error("", default, "EndTypeMismatch",
                        $"end 类型不匹配：期望 end {expectedType}，实际 end {actualType}。");

                return true;

            default:
                return false;
        }
    }


    /// <summary>
    ///     处理代码块
    /// </summary>
    private void process_code_block(OakTextReader reader, List<DejaVuTemplateNode> nodes)
    {
        var codeContent = read_code_content(reader);

        if (check_end_directive(codeContent, "", out _) != EndCheckResult.not_end)
        {
            _diagnostics.report_error("", default, "UnexpectedEnd", "此处没有需要闭合的块。");
            return;
        }

        if (codeContent.StartsWith("if "))
        {
            var condition = codeContent["if ".Length..].Trim();
            var ifNode = new DejaVuIfNode { condition = condition, parsed_condition = try_parse_expression(condition) };
            nodes.Add(ifNode);
            parse_if_block(reader, ifNode);
        }
        else if (codeContent == "raw")
        {
            var rawNode = new DejaVuRawNode();
            nodes.Add(rawNode);
            parse_block(reader, rawNode.children, "raw");
        }
        else if (codeContent.StartsWith("loop "))
        {
            var loopNode = parse_loop_directive(codeContent);
            nodes.Add(loopNode);
            parse_block(reader, loopNode.children, "loop");
        }
        else if (codeContent.StartsWith("match "))
        {
            var expression = codeContent["match ".Length..].Trim();
            var matchNode = new DejaVuMatchNode
                { expression = expression, parsed_expression = try_parse_expression(expression) };
            nodes.Add(matchNode);
            parse_block(reader, matchNode.children, "match");
        }
        else if (codeContent.StartsWith("block "))
        {
            var blockName = codeContent["block ".Length..].Trim();
            var blockNode = new DejaVuBlockNode { name = blockName };
            nodes.Add(blockNode);
            parse_block(reader, blockNode.children, "block");
        }
        else if (codeContent.StartsWith("let "))
        {
            var letContent = codeContent["let ".Length..].Trim();
            var eqIndex = letContent.IndexOf('=');
            if (eqIndex > 0)
            {
                var varName = letContent[..eqIndex].Trim();
                var expr = letContent[(eqIndex + 1)..].Trim();
                var letNode = new DejaVuLetNode
                    { variable_name = varName, expression = expr, parsed_expression = try_parse_expression(expr) };
                nodes.Add(letNode);
                parse_block(reader, letNode.children, "let");
            }
            else
            {
                nodes.Add(new DejaVuCodeNode { code = codeContent });
            }
        }
        else if (codeContent.StartsWith("with "))
        {
            var withContent = codeContent["with ".Length..].Trim();
            var eqIndex = withContent.IndexOf('=');
            if (eqIndex > 0)
            {
                var aliasName = withContent[..eqIndex].Trim();
                var expr = withContent[(eqIndex + 1)..].Trim();
                var withNode = new DejaVuWithNode
                    { alias_name = aliasName, expression = expr, parsed_expression = try_parse_expression(expr) };
                nodes.Add(withNode);
                parse_block(reader, withNode.children, "with");
            }
            else
            {
                nodes.Add(new DejaVuCodeNode { code = codeContent });
            }
        }
        else if (codeContent == "super()")
        {
            nodes.Add(new DejaVuSuperNode());
        }
        else if (codeContent.StartsWith("extends "))
        {
            var parentTemplate = codeContent["extends ".Length..].Trim();
            nodes.Add(new DejaVuExtendsNode { parent_template = parentTemplate });
        }
        else if (codeContent.StartsWith("include "))
        {
            var templatePath = codeContent["include ".Length..].Trim();
            nodes.Add(new DejaVuIncludeNode { template_path = templatePath });
        }
        else
        {
            nodes.Add(new DejaVuCodeNode { code = codeContent, parsed_expression = try_parse_expression(codeContent) });
        }
    }

    #endregion

    #region 辅助方法

    /// <summary>
    ///     解析 loop 指令（支持 loop items 和 loop item in items 两种语法）
    /// </summary>
    private DejaVuLoopNode parse_loop_directive(string codeContent)
    {
        var loopContent = codeContent["loop ".Length..].Trim();
        var inIndex = loopContent.IndexOf(" in ");
        if (inIndex > 0)
        {
            var itemName = loopContent[..inIndex].Trim();
            var expression = loopContent[(inIndex + 4)..].Trim();
            return new DejaVuLoopNode
                { item_name = itemName, expression = expression, parsed_expression = try_parse_expression(expression) };
        }

        return new DejaVuLoopNode { expression = loopContent, parsed_expression = try_parse_expression(loopContent) };
    }


    /// <summary>
    ///     安全地解析表达式，失败时返回 null
    /// </summary>
    private IExpressionNode? try_parse_expression(string expression)
    {
        if (_expression_parser == null || string.IsNullOrWhiteSpace(expression)) return null;

        try
        {
            return _expression_parser.parse(expression);
        }
        catch (ParseException)
        {
            return null;
        }
    }


    /// <summary>
    ///     解析 raw 块（所有内容作为原始文本，不解析标签）
    /// </summary>
    private void parse_raw_block(OakTextReader reader, List<DejaVuTemplateNode> nodes)
    {
        var endMarker = _language.opening_delimiter + " end " + _language.closing_delimiter;
        var endMarkerAlt = _language.opening_delimiter + " end" + _language.closing_delimiter;
        var endMarkerRaw = _language.opening_delimiter + " end raw " + _language.closing_delimiter;
        var endMarkerRawAlt = _language.opening_delimiter + " end raw" + _language.closing_delimiter;

        var remaining = reader.slice(reader.position, reader.remaining);
        var sb = new StringBuilder();

        var endIndex = remaining.IndexOf(endMarker, StringComparison.Ordinal);
        var endAltIndex = remaining.IndexOf(endMarkerAlt, StringComparison.Ordinal);

        if (endIndex < 0 || (endAltIndex >= 0 && endAltIndex < endIndex)) endIndex = endAltIndex;

        if (remaining.IndexOf(endMarkerRaw, StringComparison.Ordinal) == endIndex ||
            remaining.IndexOf(endMarkerRawAlt, StringComparison.Ordinal) == endIndex)
        {
            // end raw 也是有效的结束标记
        }

        if (endIndex >= 0)
        {
            if (endIndex > 0) sb.Append(remaining[..endIndex]);
        }
        else
        {
            sb.Append(remaining);
        }

        if (sb.Length > 0) nodes.Add(new DejaVuTextNode { text = sb.ToString() });
    }

    #endregion

    #region 块解析

    /// <summary>
    ///     解析块内容
    /// </summary>
    private void parse_block(OakTextReader reader, List<DejaVuTemplateNode> nodes, string blockType)
    {
        if (blockType == "raw")
        {
            parse_raw_block(reader, nodes);
            return;
        }

        while (!reader.is_at_end)
        {
            var text = read_until_delimiter(reader, out var isCode, out var isComment);

            if (!string.IsNullOrEmpty(text)) nodes.Add(new DejaVuTextNode { text = text });

            if (reader.is_at_end) break;

            if (isComment)
            {
                skip_comment(reader);
                continue;
            }

            if (!isCode) continue;

            var codeContent = read_code_content(reader);

            if (handle_end(codeContent, blockType)) return;

            if (reader.is_at_end && codeContent.Length > 0)
            {
                nodes.Add(new DejaVuCodeNode
                    { code = codeContent, parsed_expression = try_parse_expression(codeContent) });
                return;
            }

            if (codeContent.StartsWith("if "))
            {
                var condition = codeContent["if ".Length..].Trim();
                var ifNode = new DejaVuIfNode
                    { condition = condition, parsed_condition = try_parse_expression(condition) };
                nodes.Add(ifNode);
                parse_if_block(reader, ifNode);
            }
            else if (codeContent.StartsWith("loop "))
            {
                var loopNode = parse_loop_directive(codeContent);
                nodes.Add(loopNode);
                parse_block(reader, loopNode.children, "loop");
            }
            else if (codeContent.StartsWith("match "))
            {
                var expression = codeContent["match ".Length..].Trim();
                var matchNode = new DejaVuMatchNode
                    { expression = expression, parsed_expression = try_parse_expression(expression) };
                nodes.Add(matchNode);
                parse_block(reader, matchNode.children, "match");
            }
            else if (codeContent.StartsWith("block "))
            {
                var blockName = codeContent["block ".Length..].Trim();
                var blockNode = new DejaVuBlockNode { name = blockName };
                nodes.Add(blockNode);
                parse_block(reader, blockNode.children, "block");
            }
            else
            {
                nodes.Add(new DejaVuCodeNode
                    { code = codeContent, parsed_expression = try_parse_expression(codeContent) });
            }
        }
    }


    /// <summary>
    ///     解析 if 块（支持 else 和 else if）
    /// </summary>
    private void parse_if_block(OakTextReader reader, DejaVuIfNode ifNode)
    {
        var currentNodes = ifNode.children;

        while (!reader.is_at_end)
        {
            var text = read_until_delimiter(reader, out var isCode, out var isComment);

            if (!string.IsNullOrEmpty(text)) currentNodes.Add(new DejaVuTextNode { text = text });

            if (reader.is_at_end) break;

            if (isComment)
            {
                skip_comment(reader);
                continue;
            }

            if (!isCode) continue;

            var codeContent = read_code_content(reader);

            if (handle_end(codeContent, "if")) return;

            if (reader.is_at_end && codeContent.Length > 0)
            {
                currentNodes.Add(new DejaVuCodeNode
                    { code = codeContent, parsed_expression = try_parse_expression(codeContent) });
                return;
            }

            if (codeContent.StartsWith("else if "))
            {
                var condition = codeContent["else if ".Length..].Trim();
                var elseIfNode = new DejaVuElseIfNode
                    { condition = condition, parsed_condition = try_parse_expression(condition) };
                ifNode.else_if_nodes.Add(elseIfNode);
                currentNodes = elseIfNode.children;
            }
            else if (codeContent == "else")
            {
                currentNodes = ifNode.else_children;
            }
            else if (codeContent.StartsWith("if "))
            {
                var condition = codeContent["if ".Length..].Trim();
                var nestedIfNode = new DejaVuIfNode
                    { condition = condition, parsed_condition = try_parse_expression(condition) };
                currentNodes.Add(nestedIfNode);
                parse_if_block(reader, nestedIfNode);
            }
            else if (codeContent.StartsWith("loop "))
            {
                var loopNode = parse_loop_directive(codeContent);
                currentNodes.Add(loopNode);
                parse_block(reader, loopNode.children, "loop");
            }
            else if (codeContent.StartsWith("match "))
            {
                var expression = codeContent["match ".Length..].Trim();
                var matchNode = new DejaVuMatchNode
                    { expression = expression, parsed_expression = try_parse_expression(expression) };
                currentNodes.Add(matchNode);
                parse_block(reader, matchNode.children, "match");
            }
            else if (codeContent.StartsWith("block "))
            {
                var blockName = codeContent["block ".Length..].Trim();
                var blockNode = new DejaVuBlockNode { name = blockName };
                currentNodes.Add(blockNode);
                parse_block(reader, blockNode.children, "block");
            }
            else
            {
                currentNodes.Add(new DejaVuCodeNode
                    { code = codeContent, parsed_expression = try_parse_expression(codeContent) });
            }
        }
    }

    #endregion
}