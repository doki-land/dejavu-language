//! Dejavu tools CLI
//!
//! Library hosts (TypeScript / Python / C# / Kotlin / …) expose **library APIs only**.
//! Full editor experience (LSP) is provided by this Rust binary: `dejavu lsp`.

use clap::Parser;
use std::path::PathBuf;

/// 主函数入口
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = DejavuCli::parse();

    match cli.command {
        Commands::Build(args) => {
            BuildCommand::execute(args).await?;
        }
        Commands::Dev(args) => {
            DevCommand::execute(args).await?;
        }
        Commands::Check(args) => {
            CheckCommand::execute(args).await?;
        }
        Commands::Lsp(args) => {
            LspCommand::execute(args).await?;
        }
    }

    Ok(())
}

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

    /// 输出目录
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// 是否清理输出目录
    #[arg(short, long)]
    pub clean: bool,
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
}

/// LSP 参数
#[derive(Parser, Debug)]
pub struct LspArgs {
    /// Use stdio transport (default for editors)
    #[arg(long, default_value_t = true)]
    pub stdio: bool,
}

/// 编译命令
pub struct BuildCommand;

impl BuildCommand {
    /// 执行编译命令
    pub async fn execute(args: BuildArgs) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Building templates from {:?} to {:?}",
            args.source, args.output
        );
        println!("Clean: {}", args.clean);
        Ok(())
    }
}

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

/// 检查命令
pub struct CheckCommand;

impl CheckCommand {
    /// 执行检查命令
    pub async fn execute(args: CheckArgs) -> Result<(), Box<dyn std::error::Error>> {
        println!("Checking templates in {:?}", args.source);
        Ok(())
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
