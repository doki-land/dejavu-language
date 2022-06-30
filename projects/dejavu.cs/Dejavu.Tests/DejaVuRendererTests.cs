using System.Collections;

namespace Dejavu.Tests;

public sealed class DejaVuRendererTests
{
    [Fact]
    public void RenderBasicVariablesTest()
    {
        // Arrange
        var renderer = new DejaVuRenderer(DejaVuLanguage.Dora);
        var template = "Hello <% name %>, welcome to <% place %>!";
        var context = new Dictionary<string, object>
        {
            ["name"] = "John",
            ["place"] = "DejaVu"
        };

        // Act
        var result = renderer.Render(template, context);

        // Assert
        Assert.Equal("Hello John, welcome to DejaVu!", result);
    }

    [Fact]
    public void RenderIfStatementTest()
    {
        // Arrange
        var renderer = new DejaVuRenderer(DejaVuLanguage.Dora);
        var template = "Hello <% name %>!<% if showGreeting %> Welcome to our site!<% end if %>";
        var context = new Dictionary<string, object>
        {
            ["name"] = "John",
            ["showGreeting"] = true
        };

        // Act
        var result = renderer.Render(template, context);

        // Assert
        Assert.Equal("Hello John! Welcome to our site!", result);
    }

    [Fact]
    public void RenderIfStatementFalseTest()
    {
        // Arrange
        var renderer = new DejaVuRenderer(DejaVuLanguage.Dora);
        var template = "Hello <% name %>!<% if showGreeting %> Welcome to our site!<% end if %>";
        var context = new Dictionary<string, object>
        {
            ["name"] = "John",
            ["showGreeting"] = false
        };

        // Act
        var result = renderer.Render(template, context);

        // Assert
        Assert.Equal("Hello John!", result);
    }

    [Fact]
    public void RenderLoopStatementTest()
    {
        // Arrange
        var renderer = new DejaVuRenderer(DejaVuLanguage.Dora);
        var template = "Items:<% loop items %><% item %>,<% end loop %>";
        var context = new Dictionary<string, object>
        {
            ["items"] = new[] { "Apple", "Banana", "Cherry" }
        };

        // Act
        var result = renderer.Render(template, context);

        // Assert
        Assert.Equal("Items:Apple,Banana,Cherry,", result);
    }

    [Fact]
    public void RenderMatchStatementTest()
    {
        var renderer = new DejaVuRenderer(DejaVuLanguage.Dora);
        var template = "Match test:<% match value %>Value: <% value %><% end match %>";
        var context = new Dictionary<string, object>
        {
            ["value"] = "test"
        };
        var result = renderer.Render(template, context);
        Assert.Equal("Match test:test", result);
    }

    [Fact]
    public void RenderNestedDictionaryAccessTest()
    {
        var renderer = new DejaVuRenderer(DejaVuLanguage.Dora);
        var template =
            "<!DOCTYPE html>\n<html lang=\"<% site.language %>\">\n<title><% page.title %> - <% site.title %></title>";
        var context = new Dictionary<string, object>
        {
            ["site"] = new Dictionary<string, object>
            {
                ["title"] = "My Site",
                ["language"] = "zh-CN"
            },
            ["page"] = new Dictionary<string, object>
            {
                ["title"] = "Hello World"
            }
        };
        var result = renderer.Render(template, context);
        Assert.Contains("lang=\"zh-CN\"", result);
        Assert.Contains("<title>Hello World - My Site</title>", result);
    }

    [Fact]
    public void RenderCommentsTest()
    {
        // Arrange
        var renderer = new DejaVuRenderer(DejaVuLanguage.Dora);
        var template = "Hello <% name %><%-- This is a comment --%>!";
        var context = new Dictionary<string, object>
        {
            ["name"] = "John"
        };

        // Act
        var result = renderer.Render(template, context);

        // Assert
        Assert.Equal("Hello John!", result);
    }

    [Fact]
    public void RenderNestedStatementsTest()
    {
        // Arrange
        var renderer = new DejaVuRenderer(DejaVuLanguage.Dora);
        var template = "<% loop items %><% if item != 'Banana' %>Item: <% item %><% end if %><% end loop %>";
        var context = new Dictionary<string, object>
        {
            ["items"] = new[] { "Apple", "Banana", "Cherry" }
        };

        // Act
        var result = renderer.Render(template, context);

        // Assert
        Assert.Equal("Item: AppleItem: Cherry", result);
    }

    [Fact]
    public void RenderDokiTemplateTest()
    {
        // Arrange
        var renderer = new DejaVuRenderer(DejaVuLanguage.Doki);
        var template = "Hello {% name %}, welcome to {% place %}!";
        var context = new Dictionary<string, object>
        {
            ["name"] = "John",
            ["place"] = "DejaVu"
        };

        // Act
        var result = renderer.Render(template, context);

        // Assert
        Assert.Equal("Hello John, welcome to DejaVu!", result);
    }

    [Fact]
    public void RenderDokiIfStatementTest()
    {
        // Arrange
        var renderer = new DejaVuRenderer(DejaVuLanguage.Doki);
        var template = "Hello {% name %}!{% if showGreeting %} Welcome to our site!{% end if %}";
        var context = new Dictionary<string, object>
        {
            ["name"] = "John",
            ["showGreeting"] = true
        };

        // Act
        var result = renderer.Render(template, context);

        // Assert
        Assert.Equal("Hello John! Welcome to our site!", result);
    }

    [Fact]
    public void RenderDokiLoopStatementTest()
    {
        // Arrange
        var renderer = new DejaVuRenderer(DejaVuLanguage.Doki);
        var template = "Items:{% loop items %}{% item %},{% end loop %}";
        var context = new Dictionary<string, object>
        {
            ["items"] = new[] { "Apple", "Banana", "Cherry" }
        };

        // Act
        var result = renderer.Render(template, context);

        // Assert
        Assert.Equal("Items:Apple,Banana,Cherry,", result);
    }

    [Fact]
    public void RenderTemplateInheritanceTest()
    {
        // Arrange
        var loader = new MemoryTemplateLoader();
        loader.Add("base.dora", "<html><body><% block content %><% end block %></body></html>");
        loader.Add("page.dora", "<% extends 'base.dora' %><% block content %><h1><% title %></h1><% end block %>");

        var manager = new TemplateManager(loader);
        var renderer = new DejaVuRenderer(DejaVuLanguage.Dora, manager);

        var context = new Dictionary<string, object>
        {
            ["title"] = "Hello World"
        };

        // Act
        var result = renderer.Render("<% extends 'base.dora' %><% block content %><h1><% title %></h1><% end block %>",
            context);

        // Assert
        Assert.Equal("<html><body><h1>Hello World</h1></body></html>", result);
    }

    [Fact]
    public void RenderIncludeTest()
    {
        // Arrange
        var loader = new MemoryTemplateLoader();
        loader.Add("header.dora", "<header><% siteName %></header>");

        var manager = new TemplateManager(loader);
        var renderer = new DejaVuRenderer(DejaVuLanguage.Dora, manager);

        var context = new Dictionary<string, object>
        {
            ["siteName"] = "My Site"
        };

        // Act
        var result = renderer.Render("<% include 'header.dora' %><main>Content</main>", context);

        // Assert
        Assert.Equal("<header>My Site</header><main>Content</main>", result);
    }

    [Fact]
    public void DictionaryIsIDictionaryTest()
    {
        var dict = new Dictionary<string, object> { ["key"] = "value" };
        Assert.True(dict is IDictionary);
    }

    [Fact]
    public void ExpressionEvaluator_DictMemberAccess()
    {
        var parser = new Oak.DejaVu.Expressions.ExpressionParser();
        var ast = parser.Parse("site.language");
        Assert.IsType<Oak.DejaVu.Expressions.MemberAccessNode>(ast);

        var context = new Dictionary<string, object?>
        {
            ["site"] = new Dictionary<string, object> { ["language"] = "zh-CN" }
        };
        var evaluator = new Oak.DejaVu.Expressions.ExpressionEvaluator(context);
        var result = evaluator.Evaluate(ast);
        Assert.Equal("zh-CN", result);
    }
}