namespace Std.Data.Text.DejaVu.Optimizer;

/// <summary>
///     作用域——变量声明的命名空间
/// </summary>
public sealed class Scope
{
    private readonly Dictionary<string, SymbolKind> _declarations = new();
    private readonly Dictionary<string, bool> _references = new();


    /// <summary>
    ///     创建作用域
    /// </summary>
    public Scope(Scope? parent, string name)
    {
        this.parent = parent;
        this.name = name;
    }


    /// <summary>
    ///     作用域名称（用于调试）
    /// </summary>
    public string name { get; }


    /// <summary>
    ///     父作用域
    /// </summary>
    public Scope? parent { get; }


    /// <summary>
    ///     变量引用及其声明状态（true = 已声明）
    /// </summary>
    public IReadOnlyDictionary<string, bool> references => _references;


    /// <summary>
    ///     声明变量
    /// </summary>
    public void declare(string name, SymbolKind kind)
    {
        _declarations[name] = kind;
    }


    /// <summary>
    ///     记录变量引用（在当前作用域或父作用域中查找声明）
    /// </summary>
    public void reference(string name)
    {
        if (_references.ContainsKey(name)) return;

        var isDeclared = is_declared_in_chain(name);
        _references[name] = isDeclared;
    }


    /// <summary>
    ///     检查变量是否在当前作用域中声明
    /// </summary>
    public bool is_declared(string name)
    {
        return _declarations.ContainsKey(name);
    }


    /// <summary>
    ///     标记变量为已声明
    /// </summary>
    public void mark_declared(string name)
    {
        if (_references.ContainsKey(name)) _references[name] = true;
    }

    private bool is_declared_in_chain(string name)
    {
        var current = this;
        while (current != null)
        {
            if (current._declarations.ContainsKey(name)) return true;

            current = current.parent;
        }

        return false;
    }
}