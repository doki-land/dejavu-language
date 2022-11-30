using System.Xml.Linq;
using Std.Template.Site;

namespace Dejavu.Tests;

/// <summary>
///     SiteGenerator 集成测试与回归测试
/// </summary>
public sealed class SiteGeneratorTests : IDisposable
{
    private readonly string _tempRoot;

    public SiteGeneratorTests()
    {
        _tempRoot = Path.Combine(Path.GetTempPath(), $"DejavuTests_{Guid.NewGuid():N}");
        Directory.CreateDirectory(_tempRoot);
    }

    public void Dispose()
    {
        if (Directory.Exists(_tempRoot)) Directory.Delete(_tempRoot, true);
    }

    #region 测试 1: SiteGenerator 全管线集成测试

    [Fact]
    public void FullPipeline_GeneratesAllExpectedOutputFiles()
    {
        var sourceDir = Path.Combine(_tempRoot, "source");
        var outputDir = Path.Combine(_tempRoot, "output");

        CreateMinimalSiteFixture(sourceDir);

        var generator = new SiteGenerator(sourceDir);
        generator.Generate(sourceDir, outputDir);

        Assert.True(File.Exists(Path.Combine(outputDir, "index.html")), "首页 index.html 应存在");
        Assert.True(File.Exists(Path.Combine(outputDir, "rss.xml")), "RSS feed 应存在");
        Assert.True(File.Exists(Path.Combine(outputDir, "sitemap.xml")), "Sitemap 应存在");
        Assert.True(File.Exists(Path.Combine(outputDir, "search-index.json")), "搜索索引应存在");
        Assert.True(File.Exists(Path.Combine(outputDir, "search", "index.html")), "搜索页面应存在");

        var postDir = Path.Combine(outputDir, "posts", "hello-world");
        Assert.True(File.Exists(Path.Combine(postDir, "index.html")), "文章页面应存在");

        var post2Dir = Path.Combine(outputDir, "posts", "second-post");
        Assert.True(File.Exists(Path.Combine(post2Dir, "index.html")), "第二篇文章页面应存在");

        var indexHtml = File.ReadAllText(Path.Combine(outputDir, "index.html"));
        Assert.Contains("<html", indexHtml);
        Assert.Contains("<head>", indexHtml);
        Assert.Contains("<body", indexHtml);

        var postHtml = File.ReadAllText(Path.Combine(postDir, "index.html"));
        Assert.Contains("<html", postHtml);
        Assert.Contains("<head>", postHtml);
        Assert.Contains("<body", postHtml);
    }

    [Fact]
    public void FullPipeline_StaticFilesAreCopied()
    {
        var sourceDir = Path.Combine(_tempRoot, "source");
        var outputDir = Path.Combine(_tempRoot, "output");

        CreateMinimalSiteFixture(sourceDir);

        var generator = new SiteGenerator(sourceDir);
        generator.Generate(sourceDir, outputDir);

        Assert.True(File.Exists(Path.Combine(outputDir, "css", "style.css")), "静态 CSS 应被复制");
        Assert.True(File.Exists(Path.Combine(outputDir, "js", "main.js")), "静态 JS 应被复制");
    }

    #endregion

    #region 测试 3: RSS Feed XML Schema 验证

    [Fact]
    public void RssFeed_ContainsRequiredElements()
    {
        var sourceDir = Path.Combine(_tempRoot, "source");
        var outputDir = Path.Combine(_tempRoot, "output");

        CreateMinimalSiteFixture(sourceDir);

        var generator = new SiteGenerator(sourceDir);
        generator.Generate(sourceDir, outputDir);

        var rssPath = Path.Combine(outputDir, "rss.xml");
        Assert.True(File.Exists(rssPath), "rss.xml 应存在");

        var xml = File.ReadAllText(rssPath);

        var doc = XDocument.Parse(xml);
        Assert.NotNull(doc.Root);

        var rssElement = doc.Root;
        Assert.Equal("rss", rssElement.Name.LocalName);

        var channelElement = rssElement.Element("channel");
        Assert.NotNull(channelElement);

        var titleElement = channelElement!.Element("title");
        Assert.NotNull(titleElement);
        Assert.NotEmpty(titleElement!.Value);

        var linkElement = channelElement.Element("link");
        Assert.NotNull(linkElement);
        Assert.NotEmpty(linkElement!.Value);

        var descElement = channelElement.Element("description");
        Assert.NotNull(descElement);

        var items = channelElement.elements("item").ToList();
        Assert.NotEmpty(items);

        foreach (var item in items)
        {
            var itemTitle = item.Element("title");
            Assert.NotNull(itemTitle);
            Assert.NotEmpty(itemTitle!.Value);

            var itemLink = item.Element("link");
            Assert.NotNull(itemLink);
            Assert.NotEmpty(itemLink!.Value);

            var hasGuid = item.Element("guid") != null;
            var hasDesc = item.Element("description") != null;
            Assert.True(hasGuid || hasDesc, "每个 item 必须包含 guid 或 description");
        }
    }

    [Fact]
    public void RssFeed_IsValidXml()
    {
        var sourceDir = Path.Combine(_tempRoot, "source");
        var outputDir = Path.Combine(_tempRoot, "output");

        CreateMinimalSiteFixture(sourceDir);

        var generator = new SiteGenerator(sourceDir);
        generator.Generate(sourceDir, outputDir);

        var rssPath = Path.Combine(outputDir, "rss.xml");
        var xml = File.ReadAllText(rssPath);

        var doc = XDocument.Parse(xml);
        Assert.NotNull(doc);
        Assert.NotNull(doc.Root);
    }

    #endregion

    #region 测试 4: 分页边界测试

    [Fact]
    public void Pagination_ZeroArticles_ShowsIndexWithEmptyState()
    {
        var sourceDir = Path.Combine(_tempRoot, "source");
        var outputDir = Path.Combine(_tempRoot, "output");

        CreateSiteFixtureWithPostCount(sourceDir, 0, 3);

        var generator = new SiteGenerator(sourceDir);
        generator.Generate(sourceDir, outputDir);

        Assert.True(File.Exists(Path.Combine(outputDir, "index.html")), "即使没有文章，首页也应存在");

        var indexHtml = File.ReadAllText(Path.Combine(outputDir, "index.html"));
        Assert.Contains("<html", indexHtml);

        Assert.False(Directory.Exists(Path.Combine(outputDir, "posts", "page", "2")),
            "没有文章时不生成多余的分页");
    }

    [Fact]
    public void Pagination_ExactlyPageSize_OnlyOnePageNoNextLink()
    {
        var sourceDir = Path.Combine(_tempRoot, "source");
        var outputDir = Path.Combine(_tempRoot, "output");

        CreateSiteFixtureWithPostCount(sourceDir, 3, 3);

        var generator = new SiteGenerator(sourceDir);
        generator.Generate(sourceDir, outputDir);

        Assert.True(File.Exists(Path.Combine(outputDir, "index.html")), "首页应存在");

        Assert.False(Directory.Exists(Path.Combine(outputDir, "posts", "page", "2")),
            "正好一页时不应有第二页");

        Assert.True(File.Exists(Path.Combine(outputDir, "posts", "page", "1", "index.html")),
            "第一页应存在");

        var indexHtml = File.ReadAllText(Path.Combine(outputDir, "posts", "page", "1", "index.html"));
        Assert.DoesNotContain("下一页", indexHtml);
    }

    [Fact]
    public void Pagination_PageSizePlusOne_TwoPagesWithCorrectNavigation()
    {
        var sourceDir = Path.Combine(_tempRoot, "source");
        var outputDir = Path.Combine(_tempRoot, "output");

        CreateSiteFixtureWithPostCount(sourceDir, 4, 3);

        var generator = new SiteGenerator(sourceDir);
        generator.Generate(sourceDir, outputDir);

        Assert.True(File.Exists(Path.Combine(outputDir, "index.html")), "首页应存在");

        Assert.True(Directory.Exists(Path.Combine(outputDir, "posts", "page", "2")),
            "4 篇文章 3 条分页时应有第二页");

        Assert.True(File.Exists(Path.Combine(outputDir, "posts", "page", "1", "index.html")),
            "第一页应存在");
        Assert.True(File.Exists(Path.Combine(outputDir, "posts", "page", "2", "index.html")),
            "第二页应存在");

        Assert.False(Directory.Exists(Path.Combine(outputDir, "posts", "page", "3")),
            "只有两页不应有第三页");
    }

    #endregion

    #region 测试 5: .dora 模板回归测试

    [Fact]
    public void DoraTemplate_PreservesHtmlTagsAsIs()
    {
        var sourceDir = Path.Combine(_tempRoot, "source");
        var outputDir = Path.Combine(_tempRoot, "output");

        CreateSiteFixtureWithDoraPage(sourceDir);

        var generator = new SiteGenerator(sourceDir);
        generator.Generate(sourceDir, outputDir);

        var doraOutputDir = Path.Combine(outputDir, "pages", "script-test");
        Assert.True(File.Exists(Path.Combine(doraOutputDir, "index.html")), "dora 页面应被生成");

        var doraOutput = File.ReadAllText(Path.Combine(doraOutputDir, "index.html"));

        Assert.Contains("<script>alert('test')</script>", doraOutput);
        Assert.Contains("function test()", doraOutput);
        Assert.Contains("<div class=\"custom\">", doraOutput);

        Assert.DoesNotContain("&lt;script&gt;", doraOutput);
        Assert.DoesNotContain("&lt;div", doraOutput);
    }

    [Fact]
    public void MarkdownPage_EscapesHtmlTags()
    {
        var sourceDir = Path.Combine(_tempRoot, "source");
        var outputDir = Path.Combine(_tempRoot, "output");

        CreateSiteFixtureWithMarkdownHtmlContent(sourceDir);

        var generator = new SiteGenerator(sourceDir);
        generator.Generate(sourceDir, outputDir);

        var mdOutputDir = Path.Combine(outputDir, "posts", "html-in-md");
        Assert.True(File.Exists(Path.Combine(mdOutputDir, "index.html")), "markdown 页面应被生成");

        var mdOutput = File.ReadAllText(Path.Combine(mdOutputDir, "index.html"));
        Assert.DoesNotContain("<script>alert('xss')</script>", mdOutput);
    }

    #endregion

    #region 辅助方法：创建测试 fixtures

    /// <summary>
    ///     创建最小的站点 fixtures，包含 config、layouts、content 和 static 文件
    /// </summary>
    private static void CreateMinimalSiteFixture(string sourceDir)
    {
        Directory.CreateDirectory(sourceDir);

        File.WriteAllText(Path.Combine(sourceDir, "config.von"), @"
{
  title: ""测试站点"",
  description: ""用于集成测试的站点"",
  author: ""测试者"",
  language: ""zh-CN"",
  baseUrl: ""http://example.com"",
  paginate: 10,
  navigation: [
    {""首页"": ""/""},
    {""文章"": ""/posts/""}
  ]
}
");

        var layoutsDir = Path.Combine(sourceDir, "layouts");
        Directory.CreateDirectory(layoutsDir);

        File.WriteAllText(Path.Combine(layoutsDir, "default.dora"), @"<!DOCTYPE html>
<html lang=""zh-CN"">
<head>
    <meta charset=""UTF-8"">
    <title>测试站点</title>
</head>
<body>
    <header>测试站点</header>
    <main><% block content %><% end block %></main>
    <footer>测试者</footer>
</body>
</html>");

        File.WriteAllText(Path.Combine(layoutsDir, "index.dora"), @"<% extends 'layouts/default.dora' %>
<% block content %>
<h2>首页</h2>
<div class=""posts"">
静态内容
</div>
<% end block %>");

        File.WriteAllText(Path.Combine(layoutsDir, "post.dora"), @"<% extends 'layouts/default.dora' %>
<% block content %>
<article>
<h2>文章标题</h2>
静态内容
</article>
<% end block %>");

        File.WriteAllText(Path.Combine(layoutsDir, "search.dora"), @"<% extends 'layouts/default.dora' %>
<% block content %>
<h2>搜索</h2>
<div><input type=""text"" /></div>
<% end block %>");

        var contentDir = Path.Combine(sourceDir, "content");
        var postsDir = Path.Combine(contentDir, "posts");
        Directory.CreateDirectory(postsDir);

        File.WriteAllText(Path.Combine(postsDir, "hello-world.md"), @"---
title: Hello World
date: 2024-01-01
tags: [hello, test]
---

# Hello World

这是第一篇测试文章。

## 功能

- 快速渲染
- 易于使用
");

        File.WriteAllText(Path.Combine(postsDir, "second-post.md"), @"---
title: 第二篇文章
date: 2024-02-01
tags: [test]
---

# 第二篇文章

这是第二篇测试文章的内容。

包含一些 **粗体** 和 *斜体* 文本。
");

        var staticDir = Path.Combine(sourceDir, "static");
        var cssDir = Path.Combine(staticDir, "css");
        var jsDir = Path.Combine(staticDir, "js");
        Directory.CreateDirectory(cssDir);
        Directory.CreateDirectory(jsDir);

        File.WriteAllText(Path.Combine(cssDir, "style.css"), "body { font-family: sans-serif; }");
        File.WriteAllText(Path.Combine(jsDir, "main.js"), "console.log('test');");
    }

    /// <summary>
    ///     创建包含指定数量文章的站点 fixtures
    /// </summary>
    private static void CreateSiteFixtureWithPostCount(string sourceDir, int postCount, int paginate)
    {
        Directory.CreateDirectory(sourceDir);

        File.WriteAllText(Path.Combine(sourceDir, "config.von"), $@"
{{
  title: ""分页测试站点"",
  description: ""分页边界测试"",
  author: ""测试者"",
  language: ""zh-CN"",
  baseUrl: ""http://example.com"",
  paginate: {paginate}
}}
");

        var layoutsDir = Path.Combine(sourceDir, "layouts");
        Directory.CreateDirectory(layoutsDir);

        File.WriteAllText(Path.Combine(layoutsDir, "default.dora"), @"<!DOCTYPE html>
<html lang=""zh-CN"">
<head><meta charset=""UTF-8""><title>分页测试</title></head>
<body>
    <main><% block content %><% end block %></main>
</body>
</html>");

        File.WriteAllText(Path.Combine(layoutsDir, "index.dora"), @"<% extends 'layouts/default.dora' %>
<% block content %>
<h2>首页</h2>
<% end block %>");

        File.WriteAllText(Path.Combine(layoutsDir, "search.dora"), @"<% extends 'layouts/default.dora' %>
<% block content %>
<h2>搜索</h2>
<% end block %>");

        File.WriteAllText(Path.Combine(layoutsDir, "post.dora"), @"<% extends 'layouts/default.dora' %>
<% block content %>
<article>
<h2>文章</h2>
</article>
<% end block %>");

        var contentDir = Path.Combine(sourceDir, "content");
        var postsDir = Path.Combine(contentDir, "posts");
        Directory.CreateDirectory(postsDir);

        for (var i = 1; i <= postCount; i++)
            File.WriteAllText(Path.Combine(postsDir, $"post-{i:D3}.md"), $@"---
title: 文章 {i}
date: 2024-{i % 12 + 1:D2}-{i % 28 + 1:D2}
---

# 文章 {i}

这是第 {i} 篇测试文章。
");
    }

    /// <summary>
    ///     创建包含 .dora 模板页面的 fixtures（验证 HTML 标签保持原样）
    /// </summary>
    private static void CreateSiteFixtureWithDoraPage(string sourceDir)
    {
        Directory.CreateDirectory(sourceDir);

        File.WriteAllText(Path.Combine(sourceDir, "config.von"), @"
{
  title: ""Dora 测试站点"",
  description: ""Dora 模板回归测试"",
  author: ""测试者"",
  language: ""zh-CN"",
  baseUrl: ""http://example.com"",
  paginate: 10
}
");

        var layoutsDir = Path.Combine(sourceDir, "layouts");
        Directory.CreateDirectory(layoutsDir);

        File.WriteAllText(Path.Combine(layoutsDir, "default.dora"), @"<!DOCTYPE html>
<html lang=""zh-CN"">
<head><meta charset=""UTF-8""><title>Dora 测试</title></head>
<body>
    <main><% block content %><% end block %></main>
</body>
</html>");

        File.WriteAllText(Path.Combine(layoutsDir, "index.dora"), @"<% extends 'layouts/default.dora' %>
<% block content %>
<h2>首页</h2>
<% end block %>");

        File.WriteAllText(Path.Combine(layoutsDir, "search.dora"), @"<% extends 'layouts/default.dora' %>
<% block content %>
<h2>搜索</h2>
<% end block %>");

        File.WriteAllText(Path.Combine(layoutsDir, "pages.dora"), @"<% extends 'layouts/default.dora' %>
<% block content %>
静态页面内容
<% end block %>");

        var contentDir = Path.Combine(sourceDir, "content");
        var pagesDir = Path.Combine(contentDir, "pages");
        Directory.CreateDirectory(pagesDir);

        File.WriteAllText(Path.Combine(pagesDir, "script-test.dora"), @"---
title: Script 测试页面
date: 2024-01-01
---

<div class=""custom"">
    <h1>脚本测试</h1>
    <script>alert('test')</script>
    <pre><code>
function test() {
    console.log('hello');
}
    </code></pre>
    <p>这里的 HTML 应保持原样</p>
</div>
");
    }

    /// <summary>
    ///     创建包含 HTML 标签的 markdown 页面 fixtures
    /// </summary>
    private static void CreateSiteFixtureWithMarkdownHtmlContent(string sourceDir)
    {
        Directory.CreateDirectory(sourceDir);

        File.WriteAllText(Path.Combine(sourceDir, "config.von"), @"
{
  title: ""Markdown 测试站点"",
  description: ""Markdown HTML 保留测试"",
  author: ""测试者"",
  language: ""zh-CN"",
  baseUrl: ""http://example.com"",
  paginate: 10
}
");

        var layoutsDir = Path.Combine(sourceDir, "layouts");
        Directory.CreateDirectory(layoutsDir);

        File.WriteAllText(Path.Combine(layoutsDir, "default.dora"), @"<!DOCTYPE html>
<html lang=""zh-CN"">
<head><meta charset=""UTF-8""><title>Markdown 测试</title></head>
<body>
    <main><% block content %><% end block %></main>
</body>
</html>");

        File.WriteAllText(Path.Combine(layoutsDir, "post.dora"), @"<% extends 'layouts/default.dora' %>
<% block content %>
<article>
    <h2>Markdown HTML</h2>
    静态内容
</article>
<% end block %>");

        File.WriteAllText(Path.Combine(layoutsDir, "index.dora"), @"<% extends 'layouts/default.dora' %>
<% block content %>
<h2>首页</h2>
<% end block %>");

        File.WriteAllText(Path.Combine(layoutsDir, "search.dora"), @"<% extends 'layouts/default.dora' %>
<% block content %>
<h2>搜索</h2>
<% end block %>");

        var contentDir = Path.Combine(sourceDir, "content");
        var postsDir = Path.Combine(contentDir, "posts");
        Directory.CreateDirectory(postsDir);

        File.WriteAllText(Path.Combine(postsDir, "html-in-md.md"), @"---
title: Markdown 中的 HTML
date: 2024-03-01
---

# HTML 在 Markdown 中

正常的 Markdown 文本。

<script>alert('xss')</script>

这是一个段落。
");
    }

    #endregion
}