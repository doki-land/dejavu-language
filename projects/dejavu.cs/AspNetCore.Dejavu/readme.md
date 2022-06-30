# AspNetCore.Dejavu

ASP.NET Core integration for Dejavu. Package name uses the **AspNetCore.** host prefix.

```csharp
using AspNetCore.Dejavu;
using System.Text.Json.Nodes;

var builder = WebApplication.CreateBuilder(args);
builder.Services.AddDejavu();
var app = builder.Build();

app.MapGet("/", () =>
    DejavuResults.Html("Hello, <% name %>!", new JsonObject { ["name"] = "World" }));

// Prefer IR for cross-language identical output:
// app.MapGet("/ir", () => DejavuResults.HtmlFromIrJson(irJson, ctx));

app.Run();
```

Does not alter IR semantics; only adapts `render` output to `IResult`.
