using Std.Data.Text.DejaVu.Expressions;

namespace Std.Data.Text.DejaVu.Optimizer;

/// <summary>
///     表达式优化器——常量折叠、表达式简化
/// </summary>
public static class ExpressionOptimizer
{
    /// <summary>
    ///     优化表达式 AST，执行常量折叠等变换
    /// </summary>
    public static IExpressionNode optimize(IExpressionNode node)
    {
        return node switch
        {
            BinaryNode binary => optimize_binary(binary),
            UnaryNode unary => optimize_unary(unary),
            PipeNode pipe => optimize_pipe(pipe),
            MemberAccessNode member => optimize_member_access(member),
            CallNode call => optimize_call(call),
            IndexNode index => optimize_index(index),
            _ => node
        };
    }


    /// <summary>
    ///     优化二元表达式——常量折叠
    /// </summary>
    private static IExpressionNode optimize_binary(BinaryNode binary)
    {
        var left = optimize(binary.left);
        var right = optimize(binary.right);

        if (left is LiteralNode leftLit && right is LiteralNode rightLit)
        {
            var result = evaluate_constant_binary(leftLit.value, rightLit.value, binary.@operator);
            if (result != null) return new LiteralNode { value = result };
        }

        if (ReferenceEquals(left, binary.left) && ReferenceEquals(right, binary.right)) return binary;

        return new BinaryNode { @operator = binary.@operator, left = left, right = right };
    }


    /// <summary>
    ///     优化一元表达式——常量折叠
    /// </summary>
    private static IExpressionNode optimize_unary(UnaryNode unary)
    {
        var operand = optimize(unary.operand);

        if (operand is LiteralNode lit)
        {
            var result = evaluate_constant_unary(lit.value, unary.@operator);
            if (result != null) return new LiteralNode { value = result };
        }

        if (ReferenceEquals(operand, unary.operand)) return unary;

        return new UnaryNode { @operator = unary.@operator, operand = operand };
    }


    /// <summary>
    ///     优化管道表达式
    /// </summary>
    private static IExpressionNode optimize_pipe(PipeNode pipe)
    {
        var left = optimize(pipe.left);
        var args = pipe.arguments.Select(optimize).ToList();

        if (ReferenceEquals(left, pipe.left) && args.SequenceEqual(pipe.arguments)) return pipe;

        return new PipeNode { left = left, filter_name = pipe.filter_name, arguments = args };
    }


    /// <summary>
    ///     优化成员访问表达式
    /// </summary>
    private static IExpressionNode optimize_member_access(MemberAccessNode member)
    {
        var obj = optimize(member.@object);

        if (ReferenceEquals(obj, member.@object)) return member;

        return new MemberAccessNode { @object = obj, member_name = member.member_name };
    }


    /// <summary>
    ///     优化函数调用表达式
    /// </summary>
    private static IExpressionNode optimize_call(CallNode call)
    {
        var function = optimize(call.function);
        var args = call.arguments.Select(optimize).ToList();

        if (ReferenceEquals(function, call.function) && args.SequenceEqual(call.arguments)) return call;

        return new CallNode { function = function, arguments = args };
    }


    /// <summary>
    ///     优化索引表达式
    /// </summary>
    private static IExpressionNode optimize_index(IndexNode index)
    {
        var obj = optimize(index.@object);
        var idx = optimize(index.index);

        if (ReferenceEquals(obj, index.@object) && ReferenceEquals(idx, index.index)) return index;

        return new IndexNode { @object = obj, index = idx };
    }


    /// <summary>
    ///     常量二元运算求值
    /// </summary>
    private static object? evaluate_constant_binary(object? left, object? right, BinaryOperator op)
    {
        if (left == null || right == null) return null;

        try
        {
            if (left is string sl && right is string sr)
                return op switch
                {
                    BinaryOperator.add => sl + sr,
                    BinaryOperator.equal => sl == sr,
                    BinaryOperator.not_equal => sl != sr,
                    _ => null
                };

            if (left is bool bl && right is bool br)
                return op switch
                {
                    BinaryOperator.and => bl && br,
                    BinaryOperator.or => bl || br,
                    BinaryOperator.equal => bl == br,
                    BinaryOperator.not_equal => bl != br,
                    _ => null
                };

            var dl = Convert.ToDouble(left);
            var dr = Convert.ToDouble(right);

            return op switch
            {
                BinaryOperator.add => dl + dr,
                BinaryOperator.subtract => dl - dr,
                BinaryOperator.multiply => dl * dr,
                BinaryOperator.divide => dr != 0 ? dl / dr : null,
                BinaryOperator.modulo => dr != 0 ? dl % dr : null,
                BinaryOperator.equal => System.Math.Abs(dl - dr) < double.Epsilon,
                BinaryOperator.not_equal => System.Math.Abs(dl - dr) >= double.Epsilon,
                BinaryOperator.less_than => dl < dr,
                BinaryOperator.less_than_or_equal => dl <= dr,
                BinaryOperator.greater_than => dl > dr,
                BinaryOperator.greater_than_or_equal => dl >= dr,
                _ => null
            };
        }
        catch
        {
            return null;
        }
    }


    /// <summary>
    ///     常量一元运算求值
    /// </summary>
    private static object? evaluate_constant_unary(object? value, UnaryOperator op)
    {
        if (value == null) return null;

        try
        {
            if (value is bool b)
                return op switch
                {
                    UnaryOperator.not => !b,
                    _ => null
                };

            var d = Convert.ToDouble(value);
            return op switch
            {
                UnaryOperator.negate => -d,
                _ => null
            };
        }
        catch
        {
            return null;
        }
    }
}