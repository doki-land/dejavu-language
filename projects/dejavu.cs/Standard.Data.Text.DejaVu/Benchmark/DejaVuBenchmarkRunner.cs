using System.Diagnostics;
using System.Text;

namespace Std.Data.Text.DejaVu.Benchmark;

/// <summary>
///     DejaVu 性能基准运行器——编译速度、渲染吞吐量、代码生成质量基准。
/// </summary>
public sealed class DejaVuBenchmarkRunner
{
    /// <summary>
    ///     运行全部基准测试
    /// </summary>
    /// <returns>基准报告。</returns>
    public BenchmarkReport run_all()
    {
        var report = new BenchmarkReport
        {
            run_at = DateTimeOffset.UtcNow,
            machine_info = $"{Environment.MachineName} | {Environment.OSVersion} | .NET {Environment.Version}"
        };

        report.compile_benchmarks.AddRange(run_compile_benchmarks());
        report.render_benchmarks.AddRange(run_render_benchmarks());
        report.code_gen_benchmarks.AddRange(run_code_gen_benchmarks());

        return report;
    }


    /// <summary>
    ///     编译性能基准
    /// </summary>
    public List<BenchmarkEntry> run_compile_benchmarks()
    {
        var entries = new List<BenchmarkEntry>
        {
            benchmark("编译-简单变量", () =>
            {
                var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
                compiler.compile("{{ name }}");
            }),
            benchmark("编译-条件+循环", () =>
            {
                var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
                compiler.compile("{% if show %}{% loop item in items %}{{ item }}{% end %}{% end %}");
            }),
            benchmark("编译-管道过滤器", () =>
            {
                var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
                compiler.compile("{{ name |> uppercase |> trim |> truncate:30 }}");
            }),
            benchmark("编译-模板继承", () =>
            {
                var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
                compiler.compile("{% extends \"layout.dejavu\" %}{% block content %}Hello{% end %}");
            }),
            benchmark("编译-完整页面", () =>
            {
                var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
                compiler.compile(generate_full_page_template());
            }),
            benchmark("编译-带符号解析", () =>
            {
                var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
                compiler.compile("{% let x = 1 %}{% loop item in items %}{{ item }}{% end %}", emitSymbolTable: true);
            }),
            benchmark("编译-带类型检查", () =>
            {
                var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
                compiler.check_types("{% let x = 1 %}{% if show %}{{ x }}{% end %}");
            })
        };

        return entries;
    }


    /// <summary>
    ///     渲染性能基准
    /// </summary>
    public List<BenchmarkEntry> run_render_benchmarks()
    {
        var entries = new List<BenchmarkEntry>();
        var context = new Dictionary<string, object>
        {
            ["title"] = "Benchmark",
            ["name"] = "World",
            ["items"] = Enumerable.Range(0, 100).Select(i => $"Item {i}").ToList(),
            ["show"] = true,
            ["count"] = 42
        };

        entries.Add(benchmark("渲染-简单变量(编译+渲染)", () =>
        {
            var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
            var compiled = compiler.compile("Hello {{ name }}!", emitRenderFunc: true);
            compiled.render_func?.Invoke(context);
        }));

        entries.Add(benchmark("渲染-条件+循环(编译+渲染)", () =>
        {
            var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
            var compiled = compiler.compile("{% if show %}{% loop item in items %}{{ item }}{% end %}{% end %}",
                emitRenderFunc: true);
            compiled.render_func?.Invoke(context);
        }));

        entries.Add(benchmark("渲染-管道过滤器(编译+渲染)", () =>
        {
            var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
            var compiled = compiler.compile("{{ name |> uppercase |> trim }}", emitRenderFunc: true);
            compiled.render_func?.Invoke(context);
        }));

        entries.Add(benchmark("渲染-预编译简单变量", () =>
        {
            var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
            var compiled = compiler.compile("Hello {{ name }}!", emitRenderFunc: true);
            var renderFunc = compiled.render_func!;
            for (var i = 0; i < 10; i++) renderFunc(context);
        }));

        return entries;
    }


    /// <summary>
    ///     代码生成基准
    /// </summary>
    public List<BenchmarkEntry> run_code_gen_benchmarks()
    {
        var entries = new List<BenchmarkEntry>();
        var source = generate_full_page_template();

        entries.Add(benchmark("代码生成-TypeScript", () =>
        {
            var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
            compiler.compile_to_type_script(source);
        }));

        entries.Add(benchmark("代码生成-Java", () =>
        {
            var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
            compiler.compile_to_java(source);
        }));

        entries.Add(benchmark("代码生成-接口推导", () =>
        {
            var compiler = new DejaVuCompiler(new DejaVuParser("doki"));
            compiler.infer_type_script_interface(source);
        }));

        return entries;
    }


    /// <summary>
    ///     生成基准报告文本
    /// </summary>
    public static string generate_report(BenchmarkReport report)
    {
        var sb = new StringBuilder();
        sb.AppendLine("=== DejaVu 性能基准报告 ===");
        sb.AppendLine($"运行时间: {report.run_at:yyyy-MM-dd HH:mm:ss} UTC");
        sb.AppendLine($"运行环境: {report.machine_info}");
        sb.AppendLine();

        print_section(sb, "编译性能", report.compile_benchmarks);
        print_section(sb, "渲染性能", report.render_benchmarks);
        print_section(sb, "代码生成性能", report.code_gen_benchmarks);

        sb.AppendLine("--- 汇总 ---");
        var all = report.compile_benchmarks.Concat(report.render_benchmarks).Concat(report.code_gen_benchmarks)
            .ToList();
        sb.AppendLine($"总基准数: {all.Count}");
        sb.AppendLine($"平均耗时: {all.Average(e => e.avg_ms):F3}ms");
        sb.AppendLine($"最慢基准: {all.OrderByDescending(e => e.avg_ms).First().name} ({all.Max(e => e.avg_ms):F3}ms)");
        sb.AppendLine($"最快基准: {all.OrderBy(e => e.avg_ms).First().name} ({all.Min(e => e.avg_ms):F3}ms)");

        return sb.ToString();
    }

    private static void print_section(StringBuilder sb, string title, List<BenchmarkEntry> entries)
    {
        sb.AppendLine($"--- {title} ---");
        sb.AppendLine($"  {"名称",-30} {"迭代",6} {"平均(ms)",12} {"最小(ms)",12} {"最大(ms)",12} {"P95(ms)",12}");
        foreach (var entry in entries)
            sb.AppendLine(
                $"  {entry.name,-30} {entry.iterations,6} {entry.avg_ms,12:F3} {entry.min_ms,12:F3} {entry.max_ms,12:F3} {entry.p95_ms,12:F3}");

        sb.AppendLine();
    }

    private static BenchmarkEntry benchmark(string name, Action action, int iterations = 100)
    {
        var sw = new Stopwatch();
        var times = new List<double>();

        for (var i = 0; i < iterations; i++)
        {
            sw.Restart();
            action();
            sw.Stop();
            times.Add(sw.Elapsed.TotalMilliseconds);
        }

        times.Sort();

        return new BenchmarkEntry
        {
            name = name,
            iterations = iterations,
            avg_ms = times.Average(),
            min_ms = times[0],
            max_ms = times[^1],
            p95_ms = times[(int)(times.Count * 0.95)],
            median_ms = times[times.Count / 2]
        };
    }

    private static string generate_full_page_template()
    {
        return """
               <html>
               <head><title>{{ title }}</title></head>
               <body>
               <header>{% block header %}Default Header{% end %}</header>
               <nav>{% block nav %}{% loop item in nav_items %}<a href="{{ item.url }}">{{ item.label }}</a>{% end %}{% end %}</nav>
               <main>{% block content %}{% if show_content %}{% loop post in posts %}<article><h2>{{ post.title }}</h2><p>{{ post.body |> truncate:200 }}</p><span>{{ post.date |> date }}</span></article>{% end %}{% else %}<p>No content</p>{% end %}{% end %}</main>
               <aside>{% block sidebar %}{% let recent = recent_posts %}{% loop item in recent %}<a href="{{ item.url }}">{{ item.title }}</a>{% end %}{% end %}</aside>
               <footer>{% block footer %}&copy; 2026{% end %}</footer>
               </body>
               </html>
               """;
    }
}