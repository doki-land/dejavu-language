using System.Text;

namespace Std.Data.Text.DejaVu.Cli;

/// <summary>
///     DejaVu CLI 工具——模板预编译和项目脚手架。
/// </summary>
public sealed class DejaVuCli
{
    /// <summary>
    ///     编译模板到目标语言
    /// </summary>
    /// <param name="inputPath">输入模板路径。</param>
    /// <param name="targetLanguage">目标语言（csharp/typescript/java）。</param>
    /// <param name="outputPath">输出文件路径（可选）。</param>
    /// <param name="className">类名/函数名。</param>
    /// <returns>编译结果。</returns>
    public CompileResult compile(string inputPath, string targetLanguage, string? outputPath = null,
        string className = "TemplateRenderer")
    {
        if (!File.Exists(inputPath)) return new CompileResult(false, $"输入文件不存在: {inputPath}", string.Empty, inputPath);

        var source = File.ReadAllText(inputPath);
        var compiler = new DejaVuCompiler(new DejaVuParser("doki"));

        string output;

        try
        {
            output = targetLanguage.ToLowerInvariant() switch
            {
                "csharp" or "cs" or "c#" => compile_to_c_sharp(compiler, source, className),
                "typescript" or "ts" => compiler.compile_to_type_script(source, className),
                "java" => compiler.compile_to_java(source, className),
                _ => throw new ArgumentException($"不支持的目标语言: {targetLanguage}")
            };
        }
        catch (Exception ex)
        {
            return new CompileResult(false, $"编译失败: {ex.Message}", string.Empty, inputPath);
        }

        if (!string.IsNullOrEmpty(outputPath))
        {
            var dir = Path.GetDirectoryName(outputPath);
            if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir)) Directory.CreateDirectory(dir);

            File.WriteAllText(outputPath, output);
        }

        return new CompileResult(true, "编译成功", output, outputPath ?? inputPath);
    }


    /// <summary>
    ///     批量编译目录下的所有模板
    /// </summary>
    /// <param name="inputDir">输入目录。</param>
    /// <param name="targetLanguage">目标语言。</param>
    /// <param name="outputDir">输出目录。</param>
    /// <returns>编译结果列表。</returns>
    public List<CompileResult> compile_directory(string inputDir, string targetLanguage, string outputDir)
    {
        var results = new List<CompileResult>();

        if (!Directory.Exists(inputDir))
        {
            results.Add(new CompileResult(false, $"输入目录不存在: {inputDir}", string.Empty, inputDir));
            return results;
        }

        var extensions = new[] { ".dejavu" };
        var files = Directory.GetFiles(inputDir, "*.*", SearchOption.AllDirectories)
            .Where(f => extensions.Contains(Path.GetExtension(f).ToLowerInvariant()))
            .ToList();

        foreach (var file in files)
        {
            var relativePath = Path.GetRelativePath(inputDir, file);
            var outputExtension = get_output_extension(targetLanguage);
            var outputFileName = Path.ChangeExtension(relativePath, outputExtension);
            var outputPath = Path.Combine(outputDir, outputFileName);

            var className = Path.GetFileNameWithoutExtension(file);
            className = char.ToUpper(className[0]) + className[1..];

            results.Add(compile(file, targetLanguage, outputPath, className));
        }

        return results;
    }


    /// <summary>
    ///     初始化模板项目结构
    /// </summary>
    /// <param name="projectDir">项目目录。</param>
    /// <param name="projectName">项目名。</param>
    /// <returns>创建的文件列表。</returns>
    public List<string> init_project(string projectDir, string projectName)
    {
        var createdFiles = new List<string>();

        if (!Directory.Exists(projectDir)) Directory.CreateDirectory(projectDir);

        var dirs = new[] { "layouts", "components", "partials", "pages", "themes", "assets" };
        foreach (var dir in dirs)
        {
            var fullPath = Path.Combine(projectDir, dir);
            if (!Directory.Exists(fullPath)) Directory.CreateDirectory(fullPath);
        }

        var layoutTemplate = @"{% block title %}默认标题{% end %}
{% block content %}默认内容{% end %}
{% block footer %}<footer>© 2026</footer>{% end %}";

        var layoutPath = Path.Combine(projectDir, "layouts", "default.dejavu");
        File.WriteAllText(layoutPath, layoutTemplate);
        createdFiles.Add(layoutPath);

        var pageTemplate = @"{% extends ""layouts/default.dejavu"" %}

{% block title %}首页{% end %}

{% block content %}
<h1>欢迎使用 DejaVu</h1>
<p>这是由 dejaVu init 创建的模板项目。</p>
{% end %}";

        var pagePath = Path.Combine(projectDir, "pages", "index.dejavu");
        File.WriteAllText(pagePath, pageTemplate);
        createdFiles.Add(pagePath);

        var componentTemplate = @"<div class=""card"">
  <h2>{{ title }}</h2>
  <div>{% block body %}默认内容{% end %}</div>
</div>";

        var componentPath = Path.Combine(projectDir, "components", "card.dejavu");
        File.WriteAllText(componentPath, componentTemplate);
        createdFiles.Add(componentPath);

        var themeCss = @":root {
    --primary: #3b82f6;
    --secondary: #64748b;
    --background: #ffffff;
    --text: #1e293b;
    --border: #e2e8f0;
    --radius: 0.5rem;
}";

        var themePath = Path.Combine(projectDir, "themes", "default.css");
        File.WriteAllText(themePath, themeCss);
        createdFiles.Add(themePath);

        return createdFiles;
    }

    private static string compile_to_c_sharp(DejaVuCompiler compiler, string source, string className)
    {
        var compiled = compiler.compile(source, emitRenderFunc: true);
        if (compiled.render_func != null)
        {
            var sb = new StringBuilder();
            sb.AppendLine("// Auto-generated by DejaVu C# Code Generator");
            sb.AppendLine("// Do not edit manually.");
            sb.AppendLine();
            sb.AppendLine("using System.Collections.Generic;");
            sb.AppendLine("using System.Text;");
            sb.AppendLine();
            sb.AppendLine($"public sealed class {className}");
            sb.AppendLine("{");
            sb.AppendLine("    public string Render(IDictionary<string, object> ctx)");
            sb.AppendLine("    {");
            sb.AppendLine("        return CompiledTemplate(ctx);");
            sb.AppendLine("    }");
            sb.AppendLine("}");
            return sb.ToString();
        }

        return $"// C# 渲染委托已生成，包含 {compiled.nodes.Count} 个节点";
    }

    private static string get_output_extension(string targetLanguage)
    {
        return targetLanguage.ToLowerInvariant() switch
        {
            "csharp" or "cs" or "c#" => ".g.cs",
            "typescript" or "ts" => ".generated.ts",
            "java" => "Renderer.java",
            _ => ".generated"
        };
    }
}