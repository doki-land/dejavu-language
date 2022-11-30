using System.Collections;
using System.Diagnostics;
using System.Globalization;
using System.Text;

namespace Std.Data.Text.DejaVu.Debug;

/// <summary>
///     模板调试器——源码映射、渲染追踪、性能剖析。
/// </summary>
public sealed class TemplateDebugger
{
    private readonly Stopwatch _stopwatch = new();
    private readonly List<TraceEntry> _trace_entries = [];


    /// <summary>
    ///     是否启用渲染追踪
    /// </summary>
    public bool enable_tracing { get; init; } = true;


    /// <summary>
    ///     是否启用性能剖析
    /// </summary>
    public bool enable_profiling { get; init; } = true;


    /// <summary>
    ///     追踪条目
    /// </summary>
    public IReadOnlyList<TraceEntry> trace_entries => _trace_entries;


    /// <summary>
    ///     开始追踪
    /// </summary>
    public void start_trace()
    {
        _trace_entries.Clear();
        _stopwatch.Restart();
    }


    /// <summary>
    ///     停止追踪
    /// </summary>
    public void stop_trace()
    {
        _stopwatch.Stop();
    }


    /// <summary>
    ///     记录节点追踪
    /// </summary>
    /// <param name="nodeType">节点类型。</param>
    /// <param name="sourceLine">源码行号。</param>
    /// <param name="sourceColumn">源码列号。</param>
    /// <param name="detail">详细信息。</param>
    /// <param name="elapsedMs">耗时（毫秒）。</param>
    public void trace(string nodeType, int sourceLine, int sourceColumn, string detail, double elapsedMs = 0)
    {
        if (!enable_tracing) return;

        _trace_entries.Add(new TraceEntry
        {
            node_type = nodeType,
            source_line = sourceLine,
            source_column = sourceColumn,
            detail = detail,
            elapsed_ms = elapsedMs,
            timestamp = _stopwatch.Elapsed.TotalMilliseconds
        });
    }


    /// <summary>
    ///     生成追踪报告
    /// </summary>
    public string generate_trace_report()
    {
        var sb = new StringBuilder();
        sb.AppendLine("=== DejaVu 模板渲染追踪报告 ===");
        sb.AppendLine($"总条目数: {_trace_entries.Count}");
        sb.AppendLine($"总耗时: {_stopwatch.Elapsed.TotalMilliseconds:F2}ms");
        sb.AppendLine();

        if (_trace_entries.Count == 0)
        {
            sb.AppendLine("（无追踪数据）");
            return sb.ToString();
        }

        sb.AppendLine("--- 节点执行明细 ---");
        foreach (var entry in _trace_entries)
        {
            var location = entry.source_line > 0 ? $"L{entry.source_line}:{entry.source_column}" : "未知位置";
            var elapsed = entry.elapsed_ms > 0 ? $" [{entry.elapsed_ms:F3}ms]" : "";
            sb.AppendLine($"  [{entry.node_type}] {location} {entry.detail}{elapsed}");
        }

        if (enable_profiling)
        {
            sb.AppendLine();
            sb.AppendLine("--- 性能剖析 ---");

            var profile = _trace_entries
                .Where(e => e.elapsed_ms > 0)
                .GroupBy(e => e.node_type)
                .Select(g => new ProfileEntry
                {
                    node_type = g.Key,
                    count = g.Count(),
                    total_ms = g.Sum(e => e.elapsed_ms),
                    avg_ms = g.Average(e => e.elapsed_ms),
                    max_ms = g.Max(e => e.elapsed_ms)
                })
                .OrderByDescending(p => p.total_ms)
                .ToList();

            if (profile.Count > 0)
            {
                sb.AppendLine($"  {"类型",-15} {"次数",6} {"总耗时(ms)",12} {"平均(ms)",12} {"最大(ms)",12}");
                foreach (var p in profile)
                    sb.AppendLine(
                        $"  {p.node_type,-15} {p.count,6} {p.total_ms,12:F3} {p.avg_ms,12:F3} {p.max_ms,12:F3}");
            }
            else
            {
                sb.AppendLine("（无性能数据，启用 EnableProfiling 以收集）");
            }
        }

        return sb.ToString();
    }


    /// <summary>
    ///     生成数据上下文快照
    /// </summary>
    public static string generate_context_snapshot(IDictionary<string, object> context, int maxDepth = 3)
    {
        var sb = new StringBuilder();
        sb.AppendLine("=== 数据上下文快照 ===");

        foreach (var (key, value) in context) sb.AppendLine($"  {key}: {format_value(value, maxDepth)}");

        return sb.ToString();
    }

    private static string format_value(object? value, int depth, int indent = 0)
    {
        if (value == null) return "null";

        var prefix = new string(' ', indent * 2);

        if (depth <= 0) return $"{prefix}{value.GetType().Name}...";

        switch (value)
        {
            case string s:
                return s.Length > 50 ? $"\"{s[..50]}...\"" : $"\"{s}\"";
            case bool b:
                return b ? "true" : "false";
            case double d:
                return d.ToString(CultureInfo.InvariantCulture);
            case int i:
                return i.ToString();
            case IDictionary<string, object> dict:
                var dictSb = new StringBuilder();
                dictSb.AppendLine("{");
                foreach (var (k, v) in dict.Take(10))
                    dictSb.AppendLine($"{prefix}  {k}: {format_value(v, depth - 1, indent + 1)}");

                if (dict.Count > 10) dictSb.AppendLine($"{prefix}  ... ({dict.Count - 10} more)");

                dictSb.Append($"{prefix}}}");
                return dictSb.ToString();
            case ICollection collection:
                return $"[{collection.Count} items]";
            default:
                return value.ToString() ?? value.GetType().Name;
        }
    }
}