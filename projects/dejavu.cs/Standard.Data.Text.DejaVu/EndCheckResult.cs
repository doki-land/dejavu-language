namespace Std.Data.Text.DejaVu;

/// <summary>
///     end 闭合检查结果
/// </summary>
internal enum EndCheckResult
{
    /// <summary>
    ///     不是 end 指令
    /// </summary>
    not_end,


    /// <summary>
    ///     end 栈匹配（裸 end）
    /// </summary>
    end_stack,


    /// <summary>
    ///     end 显式匹配（end if / end loop 等）
    /// </summary>
    end_explicit
}