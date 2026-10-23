//! Dejavu tools CLI
//!
//! Library hosts (TypeScript / Python / C# / Kotlin / …) expose **library APIs only**.
//! Full editor experience (LSP) is provided by this Rust binary: `dejavu lsp`.

use clap::Parser;
use dejavu_diagnostic::Diagnostic;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::ExitCode;

use dejavu_tools::diag::{format_diagnostics, parse_error_to_diagnostic, relativize};
use dejavu_tools::scan::{ScanConfig, scan_templates};

// ---------------------------------------------------------------------------
// 入口
// ---------------------------------------------------------------------------

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = DejavuCli::parse();

    match cli.command {
        Commands::Build(args) => match BuildCommand::execute(args).await {
            Ok(exit) => exit,
            Err(e) => {
                eprintln!("dejavu build: {e}");
                ExitCode::from(2)
            }
        },
        Commands::Dev(args) => match DevCommand::execute(args).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("dejavu dev: {e}");
                ExitCode::from(2)
            }
        },
        Commands::Check(args) => match CheckCommand::execute(args).await {
            Ok(exit) => exit,
            Err(e) => {
                eprintln!("dejavu check: {e}");
                ExitCode::from(2)
            }
        },
        Commands::Lsp(args) => match LspCommand::execute(args).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("dejavu lsp: {e}");
                ExitCode::from(2)
            }
        },
    }
}

// ---------------------------------------------------------------------------
// CLI 定义
// ---------------------------------------------------------------------------

/// Dejavu CLI
#[derive(Parser, Debug)]
#[command(name = "dejavu")]
#[command(
    about = "Dejavu template engine tools (library hosts + Rust LSP binary)",
    long_about = None
)]
pub struct DejavuCli {
    /// Subcommand
    #[command(subcommand)]
    pub command: Commands,
}

/// 子命令
#[derive(Parser, Debug)]
pub enum Commands {
    /// 编译模板
    Build(BuildArgs),
    /// 开发模式（监听文件变化）
    Dev(DevArgs),
    /// 检查模板
    Check(CheckArgs),
    /// Language Server Protocol (stdio) — the only supported IDE entry
    Lsp(LspArgs),
}

/// 编译参数
#[derive(Parser, Debug)]
pub struct BuildArgs {
    /// 源目录
    #[arg(short, long)]
    pub source: Option<PathBuf>,

    /// 输出目录（当前 Rust CLI 暂不真正写文件，仅打印意图）
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// 是否清理输出目录
    #[arg(short, long)]
    pub clean: bool,

    /// 模板扩展名，逗号分隔；默认与 dejavu-napi 同步："," 表示无扩展名 + .dejavu,.html,.doki
    #[arg(long)]
    pub extensions: Option<String>,
}

/// 开发模式参数
#[derive(Parser, Debug)]
pub struct DevArgs {
    /// 源目录
    #[arg(short, long)]
    pub source: Option<PathBuf>,

    /// 端口
    #[arg(short, long, default_value = "5173")]
    pub port: u16,
}

/// 检查参数
#[derive(Parser, Debug)]
pub struct CheckArgs {
    /// 源目录
    #[arg(short, long)]
    pub source: Option<PathBuf>,

    /// 模板扩展名，逗号分隔；默认与 dejavu-napi 同步："," 表示无扩展名 + .dejavu,.html,.doki
    #[arg(long)]
    pub extensions: Option<String>,
}

/// LSP 参数
#[derive(Parser, Debug)]
pub struct LspArgs {
    /// Use stdio transport (default for editors)
    #[arg(long, default_value_t = true)]
    pub stdio: bool,
}

// ---------------------------------------------------------------------------
// 公共：扫描 + 解析 → 诊断
// ---------------------------------------------------------------------------

struct ScanResult {
    diagnostics: Vec<Diagnostic>,
    file_count: usize,
    source_map: HashMap<String, String>,
}

fn resolve_source_dir(source: Option<&PathBuf>) -> std::path::PathBuf {
    match source {
        Some(s) => s.clone(),
        None => std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
    }
}

fn run_parse_pipeline(
    source_dir: &std::path::Path,
    extensions_csv: Option<&str>,
    scan_target: Option<&std::path::Path>,
) -> Result<ScanResult, Box<dyn std::error::Error>> {
    let mut config = ScanConfig::default();
    if let Some(csv) = extensions_csv {
        config = config.with_extensions_csv(csv);
    }
    // 自动排除常见的构建产物目录
    config = config.exclude("dist").exclude("build").exclude("target");

    let files = scan_templates(source_dir, &config, scan_target)?;

    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut source_map: HashMap<String, String> = HashMap::new();
    let language = dejavu_language::default_language();

    for file in &files {
        let rel = relativize(&file.to_string_lossy(), source_dir);
        let content = std::fs::read_to_string(file)?;
        source_map.insert(rel.clone(), content.clone());

        match dejavu_language::parse_with(&content, &rel, &language) {
            Ok(_doc) => { /* 没有诊断 */ }
            Err(perr) => {
                let d = parse_error_to_diagnostic(&perr);
                diagnostics.push(d);
            }
        }
    }

    Ok(ScanResult {
        diagnostics,
        file_count: files.len(),
        source_map,
    })
}

// ---------------------------------------------------------------------------
// Build
// ---------------------------------------------------------------------------

/// 编译命令
pub struct BuildCommand;

impl BuildCommand {
    /// 执行编译命令
    pub async fn execute(args: BuildArgs) -> Result<ExitCode, Box<dyn std::error::Error>> {
        let source_dir = resolve_source_dir(args.source.as_ref());
        let source_dir = source_dir.canonicalize().unwrap_or(source_dir);

        let target = args
            .output
            .as_ref()
            .map(|o| {
                std::fs::canonicalize(o)
                    .unwrap_or_else(|_| o.clone())
            });

        let result = run_parse_pipeline(
            &source_dir,
            args.extensions.as_deref(),
            target.as_deref(),
        )?;

        // Rust CLI 不是最终 AOT 生成器 —— 这里只做校验 + 打印意图
        println!(
            "dejavu build: {} template(s) scanned",
            result.file_count
        );
        if let Some(out) = &args.output {
            println!(
                "  → would write compiled artifacts to {}",
                out.display()
            );
        }
        if args.clean {
            println!("  → would clean output directory first");
        }

        let err_text = format_diagnostics(&result.diagnostics, &result.source_map);
        if !err_text.is_empty() {
            eprint!("{err_text}");
        }

        let has_errors = result
            .diagnostics
            .iter()
            .any(|d| d.severity == dejavu_diagnostic::Severity::Error);

        if has_errors {
            println!("\ndejavu build: aborted due to parse errors");
            Ok(ExitCode::FAILURE)
        } else {
            println!("dejavu build: all templates parsed successfully");
            Ok(ExitCode::SUCCESS)
        }
    }
}

// ---------------------------------------------------------------------------
// Dev（占位）
// ---------------------------------------------------------------------------

/// 开发命令
pub struct DevCommand;

impl DevCommand {
    /// 执行开发命令
    pub async fn execute(args: DevArgs) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting dev server on port {}", args.port);
        println!("Watching files in {:?}", args.source);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Check
// ---------------------------------------------------------------------------

/// 检查命令
pub struct CheckCommand;

impl CheckCommand {
    /// 执行检查命令
    pub async fn execute(args: CheckArgs) -> Result<ExitCode, Box<dyn std::error::Error>> {
        let source_dir = resolve_source_dir(args.source.as_ref());
        let source_dir = source_dir.canonicalize().unwrap_or(source_dir);

        let result = run_parse_pipeline(
            &source_dir,
            args.extensions.as_deref(),
            None,
        )?;

        println!(
            "dejavu check: {} template(s) scanned",
            result.file_count
        );

        let err_text = format_diagnostics(&result.diagnostics, &result.source_map);
        if !err_text.is_empty() {
            eprint!("{err_text}");
        }

        let has_errors = result
            .diagnostics
            .iter()
            .any(|d| d.severity == dejavu_diagnostic::Severity::Error);

        if has_errors {
            println!("\ndejavu check: FAILED");
            Ok(ExitCode::FAILURE)
        } else {
            println!("dejavu check: all templates OK");
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// LSP command — sole supported IDE / language-server entry for all hosts.
pub struct LspCommand;

impl LspCommand {
    /// Run the language server (stdio).
    pub async fn execute(args: LspArgs) -> Result<(), Box<dyn std::error::Error>> {
        let _ = args.stdio;
        // Protocol surface is owned by this binary. Full LSP handlers land here;
        // editors must not ship parallel TypeScript/Python/… language servers.
        eprintln!(
            "dejavu lsp: language server entry is ready on this binary; \
             protocol handlers are not fully implemented yet"
        );
        Err("dejavu lsp: not fully implemented yet — install/update this Rust binary for IDE support".into())
    }
}
