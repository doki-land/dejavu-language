namespace Std.Data.Text.DejaVu.Optimizer;

/// <summary>
///     符号表——编译期收集的所有符号信息
/// </summary>
public sealed class SymbolTable
{
    private readonly List<Scope> _scopes = [];


    /// <summary>
    ///     创建符号表
    /// </summary>
    public SymbolTable(Scope globalScope)
    {
        global_scope = globalScope;
        _scopes.Add(globalScope);
    }


    /// <summary>
    ///     全局作用域
    /// </summary>
    public Scope global_scope { get; }


    /// <summary>
    ///     所有作用域
    /// </summary>
    public IReadOnlyList<Scope> all_scopes => _scopes;


    /// <summary>
    ///     父模板路径（extends 引用）
    /// </summary>
    public string? parent_template { get; set; }


    /// <summary>
    ///     所有引入的模板路径
    /// </summary>
    public List<string> included_templates { get; } = [];


    /// <summary>
    ///     所有 block 名称
    /// </summary>
    public HashSet<string> blocks { get; } = [];


    /// <summary>
    ///     添加作用域
    /// </summary>
    public void add_scope(Scope scope)
    {
        _scopes.Add(scope);
    }


    /// <summary>
    ///     注册 block 名称
    /// </summary>
    public void register_block(string name)
    {
        blocks.Add(name);
    }
}