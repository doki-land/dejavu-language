using Std.Data.Text.Diagnostics;
using Std.Data.Text.Parsing;

namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     表达式解析器，使用 Pratt Parser 算法
/// </summary>
public sealed class ExpressionParser
{
    private readonly DiagnosticSink _diagnostics;


    /// <summary>
    ///     创建表达式解析器
    /// </summary>
    /// <param name="diagnostics">诊断消息收集器。</param>
    public ExpressionParser(DiagnosticSink? diagnostics = null)
    {
        _diagnostics = diagnostics ?? new DiagnosticSink();
    }


    /// <summary>
    ///     解析表达式
    /// </summary>
    public IExpressionNode parse(string expression)
    {
        var lexer = new ExpressionLexer(expression);
        var tokens = lexer.tokenize();
        var reader = new TokenReader(tokens);
        return parse_expression(reader, 0);
    }


    /// <summary>
    ///     解析表达式（Pratt Parser 核心算法）
    /// </summary>
    private IExpressionNode parse_expression(TokenReader reader, int precedence)
    {
        if (reader.is_at_end)
        {
            _diagnostics.report_error("", default, "EmptyExpression", "Empty expression");
            return new LiteralNode { value = null };
        }

        var token = reader.advance();
        var left = parse_prefix(token, reader);

        while (!reader.is_at_end && precedence < get_precedence(reader.current.type))
        {
            token = reader.advance();
            left = parse_infix(token, left, reader);
        }

        return left;
    }


    /// <summary>
    ///     解析前缀表达式
    /// </summary>
    private IExpressionNode parse_prefix(ExpressionToken token, TokenReader reader)
    {
        return token.type switch
        {
            ExpressionTokenType.number => new LiteralNode { value = token.value },
            ExpressionTokenType.@string => new LiteralNode { value = token.value },
            ExpressionTokenType.boolean => new LiteralNode { value = token.value },
            ExpressionTokenType.identifier => parse_identifier(token, reader),
            ExpressionTokenType.minus => new UnaryNode
            {
                @operator = UnaryOperator.negate,
                operand = parse_expression(reader, get_precedence(ExpressionTokenType.minus))
            },
            ExpressionTokenType.not => new UnaryNode
            {
                @operator = UnaryOperator.not,
                operand = parse_expression(reader, get_precedence(ExpressionTokenType.not))
            },
            ExpressionTokenType.left_paren => parse_group(reader),
            _ => throw new ParseException($"Unexpected token: {token.type}")
        };
    }


    /// <summary>
    ///     解析中缀表达式
    /// </summary>
    private IExpressionNode parse_infix(ExpressionToken token, IExpressionNode left, TokenReader reader)
    {
        return token.type switch
        {
            ExpressionTokenType.plus => new BinaryNode
            {
                @operator = BinaryOperator.add,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.minus => new BinaryNode
            {
                @operator = BinaryOperator.subtract,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.multiply => new BinaryNode
            {
                @operator = BinaryOperator.multiply,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.divide => new BinaryNode
            {
                @operator = BinaryOperator.divide,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.modulo => new BinaryNode
            {
                @operator = BinaryOperator.modulo,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.equal => new BinaryNode
            {
                @operator = BinaryOperator.equal,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.not_equal => new BinaryNode
            {
                @operator = BinaryOperator.not_equal,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.less_than => new BinaryNode
            {
                @operator = BinaryOperator.less_than,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.less_than_or_equal => new BinaryNode
            {
                @operator = BinaryOperator.less_than_or_equal,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.greater_than => new BinaryNode
            {
                @operator = BinaryOperator.greater_than,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.greater_than_or_equal => new BinaryNode
            {
                @operator = BinaryOperator.greater_than_or_equal,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.and => new BinaryNode
            {
                @operator = BinaryOperator.and,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.or => new BinaryNode
            {
                @operator = BinaryOperator.or,
                left = left,
                right = parse_expression(reader, get_precedence(token.type))
            },
            ExpressionTokenType.pipe => parse_pipe(left, reader),
            ExpressionTokenType.dot => parse_member_access(left, reader),
            ExpressionTokenType.left_paren => parse_call(left, reader),
            ExpressionTokenType.left_bracket => parse_index(left, reader),
            _ => throw new ParseException($"Unexpected token: {token.type}")
        };
    }


    /// <summary>
    ///     解析标识符
    /// </summary>
    private IExpressionNode parse_identifier(ExpressionToken token, TokenReader reader)
    {
        if (reader is { is_at_end: false, current.type: ExpressionTokenType.left_paren })
            return parse_call(new IdentifierNode { name = token.value?.ToString() ?? "" }, reader);

        return new IdentifierNode { name = token.value?.ToString() ?? "" };
    }


    /// <summary>
    ///     解析分组表达式
    /// </summary>
    private IExpressionNode parse_group(TokenReader reader)
    {
        var expr = parse_expression(reader, 0);
        if (reader.is_at_end || reader.current.type != ExpressionTokenType.right_paren)
            _diagnostics.report_error("", default, "MissingClosingParen", "Missing closing parenthesis");
        else
            reader.advance();

        return expr;
    }


    /// <summary>
    ///     解析成员访问
    /// </summary>
    private IExpressionNode parse_member_access(IExpressionNode left, TokenReader reader)
    {
        if (reader.is_at_end || reader.current.type != ExpressionTokenType.identifier)
        {
            _diagnostics.report_error("", default, "MissingMemberName", "Missing member name after dot");
            return left;
        }

        var memberName = reader.advance().value?.ToString() ?? "";
        return new MemberAccessNode { @object = left, member_name = memberName };
    }


    /// <summary>
    ///     解析函数调用
    /// </summary>
    private IExpressionNode parse_call(IExpressionNode left, TokenReader reader)
    {
        var arguments = new List<IExpressionNode>();
        reader.advance(); // 跳过左括号

        while (!reader.is_at_end && reader.current.type != ExpressionTokenType.right_paren)
        {
            arguments.Add(parse_expression(reader, 0));
            if (reader is { is_at_end: false, current.type: ExpressionTokenType.comma }) reader.advance();
        }

        if (reader.is_at_end || reader.current.type != ExpressionTokenType.right_paren)
            _diagnostics.report_error("", default, "MissingClosingParen",
                "Missing closing parenthesis in function call");
        else
            reader.advance();

        return new CallNode { function = left, arguments = arguments };
    }


    /// <summary>
    ///     解析索引访问
    /// </summary>
    private IExpressionNode parse_index(IExpressionNode left, TokenReader reader)
    {
        reader.advance(); // 跳过左括号
        var index = parse_expression(reader, 0);

        if (reader.is_at_end || reader.current.type != ExpressionTokenType.right_bracket)
            _diagnostics.report_error("", default, "MissingClosingBracket", "Missing closing bracket");
        else
            reader.advance();

        return new IndexNode { @object = left, index = index };
    }


    /// <summary>
    ///     解析管道表达式
    /// </summary>
    private IExpressionNode parse_pipe(IExpressionNode left, TokenReader reader)
    {
        if (reader.is_at_end || reader.current.type != ExpressionTokenType.identifier)
        {
            _diagnostics.report_error("", default, "MissingFilterName", "Missing filter name after |>");
            return left;
        }

        var filterName = reader.advance().value?.ToString() ?? "";
        var arguments = new List<IExpressionNode>();

        if (reader is { is_at_end: false, current.type: ExpressionTokenType.colon })
        {
            reader.advance();
            while (!reader.is_at_end && reader.current.type != ExpressionTokenType.pipe)
            {
                arguments.Add(parse_expression(reader, 0));
                if (reader is { is_at_end: false, current.type: ExpressionTokenType.comma }) reader.advance();
            }
        }

        var pipeNode = new PipeNode { left = left, filter_name = filterName, arguments = arguments };
        return pipeNode;
    }


    /// <summary>
    ///     获取运算符优先级
    /// </summary>
    private int get_precedence(ExpressionTokenType tokenType)
    {
        return tokenType switch
        {
            ExpressionTokenType.pipe => 5,
            ExpressionTokenType.or => 10,
            ExpressionTokenType.and => 20,
            ExpressionTokenType.equal => 30,
            ExpressionTokenType.not_equal => 30,
            ExpressionTokenType.less_than => 40,
            ExpressionTokenType.less_than_or_equal => 40,
            ExpressionTokenType.greater_than => 40,
            ExpressionTokenType.greater_than_or_equal => 40,
            ExpressionTokenType.plus => 50,
            ExpressionTokenType.minus => 50,
            ExpressionTokenType.multiply => 60,
            ExpressionTokenType.divide => 60,
            ExpressionTokenType.modulo => 60,
            ExpressionTokenType.dot => 70,
            ExpressionTokenType.left_paren => 80,
            ExpressionTokenType.left_bracket => 80,
            _ => 0
        };
    }
}