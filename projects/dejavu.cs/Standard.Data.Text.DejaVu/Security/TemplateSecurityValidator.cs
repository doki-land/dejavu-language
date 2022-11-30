using System.Reflection;

namespace Std.Data.Text.DejaVu.Security;

/// <summary>
///     模板安全验证器
/// </summary>
public sealed class TemplateSecurityValidator
{
    private readonly TemplateSecurityOptions _options;

    public TemplateSecurityValidator(TemplateSecurityOptions? options = null)
    {
        _options = options ?? new TemplateSecurityOptions();
    }


    /// <summary>
    ///     验证类型访问
    /// </summary>
    public bool validate_type_access(Type type)
    {
        if (!_options.enable_sandbox) return true;

        if (_options.blocked_types.Contains(type)) return false;

        if (_options.allowed_types.Count > 0 && !_options.allowed_types.Contains(type)) return false;

        return true;
    }


    /// <summary>
    ///     验证方法调用
    /// </summary>
    public bool validate_method_call(MethodInfo method)
    {
        if (!_options.enable_sandbox) return true;

        var methodName = method.Name;

        if (_options.blocked_methods.Contains(methodName)) return false;

        if (_options.allowed_methods.Count > 0 && !_options.allowed_methods.Contains(methodName)) return false;

        return true;
    }


    /// <summary>
    ///     验证循环迭代次数
    /// </summary>
    public bool validate_loop_iteration(int currentIteration)
    {
        if (!_options.enable_sandbox) return true;

        return currentIteration < _options.max_loop_iterations;
    }
}