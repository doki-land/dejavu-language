namespace Std.Data.Text.DejaVu.Optimizer;

/// <summary>
///     符号类型
/// </summary>
public enum SymbolKind
{
    /// <summary>
    ///     迭代变量（loop item）
    /// </summary>
    iteration_variable,


    /// <summary>
    ///     局部变量（let 绑定）
    /// </summary>
    local_variable,


    /// <summary>
    ///     作用域别名（with 绑定）
    /// </summary>
    scope_alias
}