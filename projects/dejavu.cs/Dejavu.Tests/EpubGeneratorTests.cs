using System.IO.Compression;
using System.Text;
using Std.Template.EBook;

namespace Dejavu.Tests;

/// <summary>
///     EpubGenerator EPUB 输出验证测试
/// </summary>
public sealed class EpubGeneratorTests : IDisposable
{
    private readonly string _tempRoot;

    public EpubGeneratorTests()
    {
        _tempRoot = Path.Combine(Path.GetTempPath(), $"DejavuEpubTests_{Guid.NewGuid():N}");
        Directory.CreateDirectory(_tempRoot);
    }

    public void Dispose()
    {
        if (Directory.Exists(_tempRoot)) Directory.Delete(_tempRoot, true);
    }

    #region 测试 2: EpubGenerator 输出验证

    [Fact]
    public void GenerateEpub_OutputZipContainsRequiredFiles()
    {
        var outputPath = Path.Combine(_tempRoot, "test-book.epub");

        var metadata = new EBookMetadata
        {
            Title = "测试电子书",
            Author = "测试作者",
            Language = "zh-CN",
            Description = "用于测试的 EPUB 电子书",
            Publisher = "测试出版社"
        };

        var chapters = new List<EBookChapter>
        {
            new()
            {
                Title = "第一章 引言",
                HtmlContent = "<h1>第一章 引言</h1><p>这是第一章的内容。</p>",
                Order = 1
            },
            new()
            {
                Title = "第二章 深入",
                HtmlContent = "<h1>第二章 深入</h1><p>这是第二章的内容，包含<strong>重要</strong>信息。</p>",
                Order = 2
            },
            new()
            {
                Title = "第三章 总结",
                HtmlContent = "<h1>第三章 总结</h1><p>这是总结章节。</p><pre><code>var x = 1;</code></pre>",
                Order = 3
            }
        };

        var generator = new EpubGenerator();
        var result = generator.Generate(metadata, chapters, outputPath);

        Assert.True(File.Exists(result.OutputPath), "EPUB 文件应被创建");
        Assert.True(result.FileSizeBytes > 0, "EPUB 文件不应为空");
        Assert.Equal(3, result.ChapterCount);
        Assert.Equal("epub", result.Format);

        using var zipStream = new FileStream(result.OutputPath, FileMode.Open, FileAccess.Read);
        using var archive = new ZipArchive(zipStream, ZipArchiveMode.Read);

        var entries = archive.Entries.Select(e => e.FullName).ToList();

        Assert.Contains(entries, e => e == "mimetype");

        Assert.Contains(entries, e => e.StartsWith("META-INF/") && e.EndsWith(".xml"));

        Assert.Contains(entries, e => e.StartsWith("OEBPS/content_") && e.EndsWith(".opf"));

        Assert.Contains(entries, e => e == "OEBPS/nav.xhtml");

        Assert.Contains(entries, e => e == "OEBPS/styles/style.css");
    }

    [Fact]
    public void GenerateEpub_MimetypeIsUncompressedAndCorrect()
    {
        var outputPath = Path.Combine(_tempRoot, "book.epub");

        var metadata = new EBookMetadata
        {
            Title = "Mimetype 测试",
            Author = "作者"
        };

        var chapters = new List<EBookChapter>
        {
            new()
            {
                Title = "第一章",
                HtmlContent = "<h1>第一章</h1><p>内容</p>",
                Order = 1
            }
        };

        var generator = new EpubGenerator();
        generator.Generate(metadata, chapters, outputPath);

        using var zipStream = new FileStream(outputPath, FileMode.Open, FileAccess.Read);
        using var archive = new ZipArchive(zipStream, ZipArchiveMode.Read);

        var mimetypeEntry = archive.GetEntry("mimetype");
        Assert.NotNull(mimetypeEntry);

        using var reader = new StreamReader(mimetypeEntry!.Open(), Encoding.ASCII);
        var mimetypeContent = reader.ReadToEnd();
        Assert.Equal("application/epub+zip", mimetypeContent);

        Assert.True(mimetypeEntry.Length > 0, "mimetype 文件不应为空");
    }

    [Fact]
    public void GenerateEpub_ContainerXmlExistsAndIsValid()
    {
        var outputPath = Path.Combine(_tempRoot, "container-test.epub");

        var metadata = new EBookMetadata
        {
            Title = "Container 测试",
            Author = "测试者"
        };

        var chapters = new List<EBookChapter>
        {
            new()
            {
                Title = "单章",
                HtmlContent = "<h1>单章</h1><p>只有一章。</p>",
                Order = 1
            }
        };

        var generator = new EpubGenerator();
        generator.Generate(metadata, chapters, outputPath);

        using var zipStream = new FileStream(outputPath, FileMode.Open, FileAccess.Read);
        using var archive = new ZipArchive(zipStream, ZipArchiveMode.Read);

        var containerEntry =
            archive.Entries.FirstOrDefault(e => e.FullName.StartsWith("META-INF/") && e.FullName.EndsWith(".xml"));
        Assert.NotNull(containerEntry);

        using var reader = new StreamReader(containerEntry!.Open(), Encoding.UTF8);
        var containerXml = reader.ReadToEnd();

        Assert.Contains("container", containerXml);
        Assert.Contains("rootfiles", containerXml);
        Assert.Contains("OEBPS/content_", containerXml);
    }

    [Fact]
    public void GenerateEpub_ContentOpfHasCorrectStructure()
    {
        var outputPath = Path.Combine(_tempRoot, "opf-test.epub");

        var metadata = new EBookMetadata
        {
            Title = "OPF 测试",
            Author = "测试者",
            Language = "zh-CN",
            Description = "OPF 结构验证"
        };

        var chapters = new List<EBookChapter>
        {
            new()
            {
                Title = "第一章",
                HtmlContent = "<h1>第一章</h1><p>内容</p>",
                Order = 1
            }
        };

        var generator = new EpubGenerator();
        generator.Generate(metadata, chapters, outputPath);

        using var zipStream = new FileStream(outputPath, FileMode.Open, FileAccess.Read);
        using var archive = new ZipArchive(zipStream, ZipArchiveMode.Read);

        var opfEntry = archive.Entries.FirstOrDefault(e =>
            e.FullName.StartsWith("OEBPS/content_") && e.FullName.EndsWith(".opf"));
        Assert.NotNull(opfEntry);

        using var reader = new StreamReader(opfEntry!.Open(), Encoding.UTF8);
        var opfContent = reader.ReadToEnd();

        Assert.Contains("<package", opfContent);
        Assert.Contains("<metadata", opfContent);
        Assert.Contains("<manifest", opfContent);
        Assert.Contains("<spine", opfContent);

        Assert.Contains("测试者", opfContent);
        Assert.Contains("OPF 测试", opfContent);
    }

    [Fact]
    public void GenerateEpub_ChapterFilesAreXhtml()
    {
        var outputPath = Path.Combine(_tempRoot, "xhtml-test.epub");

        var metadata = new EBookMetadata
        {
            Title = "XHTML 测试",
            Author = "测试者"
        };

        var chapters = new List<EBookChapter>
        {
            new()
            {
                Title = "测试章节",
                HtmlContent = "<h2>小节</h2><p>段落内容</p>",
                Order = 1
            }
        };

        var generator = new EpubGenerator();
        generator.Generate(metadata, chapters, outputPath);

        using var zipStream = new FileStream(outputPath, FileMode.Open, FileAccess.Read);
        using var archive = new ZipArchive(zipStream, ZipArchiveMode.Read);

        var chapterEntry = archive.Entries.FirstOrDefault(e =>
            e.FullName.StartsWith("OEBPS/chapter-") && e.FullName.EndsWith(".xhtml"));
        Assert.NotNull(chapterEntry);

        using var reader = new StreamReader(chapterEntry!.Open(), Encoding.UTF8);
        var xhtml = reader.ReadToEnd();

        Assert.Contains("<!DOCTYPE html>", xhtml);
        Assert.Contains("<html xmlns=\"http://www.w3.org/1999/xhtml\"", xhtml);
        Assert.Contains("测试章节", xhtml);
    }

    [Fact]
    public void GenerateEpub_EmptyChaptersProducesValidEpub()
    {
        var outputPath = Path.Combine(_tempRoot, "empty-book.epub");

        var metadata = new EBookMetadata
        {
            Title = "空书籍",
            Author = "测试者"
        };

        var chapters = new List<EBookChapter>();

        var generator = new EpubGenerator();
        var result = generator.Generate(metadata, chapters, outputPath);

        Assert.True(File.Exists(result.OutputPath), "即使没有章节，EPUB 也应有效生成");
        Assert.Equal(0, result.ChapterCount);
        Assert.True(result.FileSizeBytes > 0, "空 EPUB 文件不应为空");
    }

    #endregion
}