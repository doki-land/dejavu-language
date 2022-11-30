namespace Std.Data.Text.DejaVu.Optimizer;

/// <summary>
///     类型环境——变量作用域 + 类型绑定
/// </summary>
public sealed class TypeEnvironment
{
    private readonly Stack<Dictionary<string, TemplateType>> _scopes = new();


    /// <summary>
    ///     创建类型环境
    /// </summary>
    public TypeEnvironment(Dictionary<string, TemplateType>? knownTypes)
    {
        var globalScope = new Dictionary<string, TemplateType>();
        if (knownTypes != null)
            foreach (var (name, type) in knownTypes)
            {
                globalScope[name] = type;
                inferred_types[name] = type;
            }

        _scopes.Push(globalScope);
    }


    /// <summary>
    ///     推导出的变量类型表
    /// </summary>
    public Dictionary<string, TemplateType> inferred_types { get; } = new();


    /// <summary>
    ///     声明变量类型
    /// </summary>
    public void declare(string name, TemplateType type)
    {
        _scopes.Peek()[name] = type;
        inferred_types[name] = type;
    }


    /// <summary>
    ///     推导变量类型（仅在未声明时设置）
    /// </summary>
    public void infer_type(string name, TemplateType type)
    {
        inferred_types.TryAdd(name, type);
    }


    /// <summary>
    ///     查找变量类型
    /// </summary>
    public bool try_get_type(string name, out TemplateType type)
    {
        foreach (var scope in _scopes)
            if (scope.TryGetValue(name, out type))
                return true;

        type = TemplateType.unknown;
        return false;
    }


    /// <summary>
    ///     推入新作用域
    /// </summary>
    public void push_scope()
    {
        _scopes.Push(new Dictionary<string, TemplateType>());
    }


    /// <summary>
    ///     弹出作用域
    /// </summary>
    public void pop_scope()
    {
        if (_scopes.Count > 1) _scopes.Pop();
    }
}