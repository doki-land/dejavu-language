namespace Dejavu.Language;

/// <summary>Span-aware parse failure.</summary>
public sealed class ParseError : Exception
{
    public string File { get; }
    public int Start { get; }
    public int Length { get; }
    public string Label { get; }

    public ParseError(
        string message,
        int start,
        int length = 1,
        string? file = null,
        string? label = null)
        : base(message)
    {
        File = file ?? "template.dejavu";
        Start = start;
        Length = Math.Max(1, length);
        Label = label ?? "here";
    }
}
