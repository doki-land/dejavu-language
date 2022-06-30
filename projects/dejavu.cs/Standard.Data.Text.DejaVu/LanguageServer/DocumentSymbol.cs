namespace Std.Data.Text.DejaVu.LanguageServer;

/// <summary>
///     文档符号
/// </summary>
public sealed class DocumentSymbol
{
    /// <summary>
    ///     符号名称
    /// </summary>
    public string name { get; init; } = string.Empty;


    /// <summary>
    ///     符号类型（1=File, 2=Module, 3=Namespace, 4=Package, 5=Class, 6=Method, 7=Property, 8=Field, 9=Constructor, 10=Enum,
    ///     11=Interface, 12=Function, 13=Variable, 14=Constant, 15=String, 16=Number, 17=Boolean, 18=Array, 19=Object, 20=Key,
    ///     21=Null, 22=EnumMember, 23=Struct, 24=Event, 25=Operator, 26=TypeParameter）
    /// </summary>
    public int kind { get; init; }


    /// <summary>
    ///     符号详情
    /// </summary>
    public string detail { get; init; } = string.Empty;
}