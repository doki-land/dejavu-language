using System.Text.Json.Nodes;
using Xunit;
using Dj = Dejavu.Dejavu;

namespace Dejavu.Tests.Ir;

public class ConformanceTests
{
    static string Root =>
        Path.GetFullPath(Path.Combine(
            AppContext.BaseDirectory, "..", "..", "..", "..", "..", "..",
            "specifications", "conformance", "t1"));

    [Fact]
    public void AllT1Cases()
    {
        Assert.True(Directory.Exists(Root), $"missing {Root}");
        foreach (var dir in Directory.GetDirectories(Root).OrderBy(d => d))
        {
            var name = Path.GetFileName(dir);
            var input = File.ReadAllText(Path.Combine(dir, "input.dejavu"));
            var expectedIr = File.ReadAllText(Path.Combine(dir, "expected.ir.json"));
            var ctx = JsonNode.Parse(File.ReadAllText(Path.Combine(dir, "context.ctx.json")))!.AsObject();
            var expectedOut = File.ReadAllText(Path.Combine(dir, "expected.out.txt"));

            var got = Dj.Parse(input);
            var normGot = Dj.NormalizeIrJson(got.ToJsonString());
            var normExp = Dj.NormalizeIrJson(expectedIr);
            Assert.True(normGot == normExp, $"IR mismatch in {name}\n{normGot}\n!=\n{normExp}");

            var outText = Dj.Render(JsonNode.Parse(expectedIr)!.AsObject(), ctx);
            Assert.Equal(expectedOut, outText);
        }
    }
}
