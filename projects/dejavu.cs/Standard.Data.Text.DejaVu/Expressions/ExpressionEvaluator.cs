using System.Collections;
using Std.Data.Text.DejaVu.Filters;

namespace Std.Data.Text.DejaVu.Expressions;

/// <summary>
///     表达式求值器
/// </summary>
public sealed class ExpressionEvaluator
{
    private readonly FilterRegistry? _filters;
    private readonly Dictionary<string, object?> _variables;


    /// <summary>
    ///     创建表达式求值器
    /// </summary>
    /// <param name="variables">变量表。</param>
    /// <param name="filters">过滤器注册表。</param>
    public ExpressionEvaluator(Dictionary<string, object?>? variables = null, FilterRegistry? filters = null)
    {
        _variables = variables ?? new Dictionary<string, object?>();
        _filters = filters;
    }


    /// <summary>
    ///     设置变量
    /// </summary>
    public void set_variable(string name, object? value)
    {
        _variables[name] = value;
    }


    /// <summary>
    ///     求值表达式
    /// </summary>
    public object? evaluate(IExpressionNode node)
    {
        return node switch
        {
            LiteralNode literal => literal.value,
            IdentifierNode identifier => get_variable(identifier.name),
            BinaryNode binary => evaluate_binary(binary),
            UnaryNode unary => evaluate_unary(unary),
            MemberAccessNode memberAccess => evaluate_member_access(memberAccess),
            CallNode call => evaluate_call(call),
            IndexNode index => evaluate_index(index),
            PipeNode pipe => evaluate_pipe(pipe),
            _ => null
        };
    }


    /// <summary>
    ///     求值二元表达式
    /// </summary>
    private object? evaluate_binary(BinaryNode binary)
    {
        var left = evaluate(binary.left);
        var right = evaluate(binary.right);

        return binary.@operator switch
        {
            BinaryOperator.add => add(left, right),
            BinaryOperator.subtract => subtract(left, right),
            BinaryOperator.multiply => multiply(left, right),
            BinaryOperator.divide => divide(left, right),
            BinaryOperator.modulo => modulo(left, right),
            BinaryOperator.equal => equal(left, right),
            BinaryOperator.not_equal => !equal(left, right),
            BinaryOperator.less_than => compare(left, right) < 0,
            BinaryOperator.less_than_or_equal => compare(left, right) <= 0,
            BinaryOperator.greater_than => compare(left, right) > 0,
            BinaryOperator.greater_than_or_equal => compare(left, right) >= 0,
            BinaryOperator.and => to_boolean(left) && to_boolean(right),
            BinaryOperator.or => to_boolean(left) || to_boolean(right),
            _ => null
        };
    }


    /// <summary>
    ///     求值一元表达式
    /// </summary>
    private object? evaluate_unary(UnaryNode unary)
    {
        var operand = evaluate(unary.operand);

        return unary.@operator switch
        {
            UnaryOperator.negate => negate(operand),
            UnaryOperator.not => !to_boolean(operand),
            _ => null
        };
    }


    /// <summary>
    ///     求值成员访问
    /// </summary>
    private object? evaluate_member_access(MemberAccessNode memberAccess)
    {
        var obj = evaluate(memberAccess.@object);
        if (obj == null) return null;

        if (obj is IDictionary dict)
            return dict.Contains(memberAccess.member_name) ? dict[memberAccess.member_name] : null;

        var type = obj.GetType();
        var property = type.GetProperty(memberAccess.member_name);
        if (property != null) return property.GetValue(obj);

        var field = type.GetField(memberAccess.member_name);
        if (field != null) return field.GetValue(obj);

        return null;
    }


    /// <summary>
    ///     求值函数调用
    /// </summary>
    private object? evaluate_call(CallNode call)
    {
        var function = evaluate(call.function);
        var arguments = call.arguments.Select(evaluate).ToArray();

        if (function is Delegate delegateFunc) return delegateFunc.DynamicInvoke(arguments);

        return null;
    }


    /// <summary>
    ///     求值索引访问
    /// </summary>
    private object? evaluate_index(IndexNode index)
    {
        var obj = evaluate(index.@object);
        var idx = evaluate(index.index);

        if (obj is IList list && idx is int i) return list[i];

        if (obj is IDictionary dict && idx != null) return dict[idx];

        return null;
    }


    /// <summary>
    ///     求值管道表达式
    /// </summary>
    private object? evaluate_pipe(PipeNode pipe)
    {
        var value = evaluate(pipe.left);

        if (_filters == null) return value;

        var args = pipe.arguments.Select(evaluate).ToArray();
        return _filters.apply(pipe.filter_name, value, args);
    }

    private object? get_variable(string name)
    {
        return _variables.GetValueOrDefault(name);
    }

    private static object? add(object? left, object? right)
    {
        if (left is string || right is string) return $"{left}{right}";

        if (left is double d1 && right is double d2) return d1 + d2;

        return null;
    }

    private static object? subtract(object? left, object? right)
    {
        if (left is double d1 && right is double d2) return d1 - d2;

        return null;
    }

    private static object? multiply(object? left, object? right)
    {
        if (left is double d1 && right is double d2) return d1 * d2;

        return null;
    }

    private static object? divide(object? left, object? right)
    {
        if (left is double d1 && right is double d2 && d2 != 0) return d1 / d2;

        return null;
    }

    private static object? modulo(object? left, object? right)
    {
        if (left is double d1 && right is double d2 && d2 != 0) return d1 % d2;

        return null;
    }

    private static bool equal(object? left, object? right)
    {
        if (left == null && right == null) return true;

        if (left == null || right == null) return false;

        return left.Equals(right);
    }

    private static int compare(object? left, object? right)
    {
        if (left is double d1 && right is double d2) return d1.CompareTo(d2);

        if (left is string s1 && right is string s2) return string.Compare(s1, s2, StringComparison.Ordinal);

        return 0;
    }

    private static bool to_boolean(object? value)
    {
        if (value == null) return false;

        if (value is bool b) return b;

        if (value is double d) return d != 0;

        if (value is string s) return !string.IsNullOrEmpty(s);

        return true;
    }

    private static object? negate(object? value)
    {
        if (value is double d) return -d;

        return null;
    }
}