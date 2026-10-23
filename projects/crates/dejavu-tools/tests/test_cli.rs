//! 端到端测试：scan fixture + 构造 parse error 验证 exit code 1。
//!
//! 通过 `cargo run` 调用二进制。为了加速，使用 `CARGO_BIN_EXE_dejavu` 环境变量。

use std::path::{Path, PathBuf};
use std::process::Command;

/// 找到 crate 根的 fixtures 目录
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("check-fixture")
}

fn run_dejavu(args: &[&str]) -> std::process::Output {
    // 优先用已编译的二进制（通过测试 harness 设置 CARGO_BIN_EXE_dejavu），
    // 否则回退到 `cargo run`。
    if let Some(exe) = std::env::var_os("CARGO_BIN_EXE_dejavu") {
        Command::new(exe).args(args).output().unwrap()
    } else {
        Command::new(env!("CARGO"))
            .args(["run", "--bin", "dejavu", "--", "--"])
            .args(args)
            .output()
            .unwrap()
    }
}

// ---------------------------------------------------------------------------
// check 命令
// ---------------------------------------------------------------------------

#[test]
fn check_valid_fixture_only_passes() {
    let fixtures = fixtures_dir();
    let src = fixtures.join("valid-only");
    let _ = std::fs::remove_dir_all(&src);
    std::fs::create_dir_all(&src).unwrap();
    std::fs::copy(fixtures.join("valid.dejavu"), src.join("valid.dejavu")).unwrap();

    let out = run_dejavu(&["check", "-s", src.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "check should pass for valid template\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1 template(s) scanned"));
    assert!(stdout.contains("all templates OK"));
}

#[test]
fn check_with_broken_fails_with_exit_1() {
    let fixtures = fixtures_dir();
    let src = fixtures.join("mixed");
    let _ = std::fs::remove_dir_all(&src);
    std::fs::create_dir_all(&src).unwrap();
    std::fs::copy(fixtures.join("valid.dejavu"), src.join("valid.dejavu")).unwrap();
    std::fs::copy(fixtures.join("broken.dejavu"), src.join("broken.dejavu")).unwrap();

    let out = run_dejavu(&["check", "-s", src.to_str().unwrap()]);
    assert!(
        !out.status.success(),
        "check should fail on broken template\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error["));
    assert!(stderr.contains("broken.dejavu"));
}

// ---------------------------------------------------------------------------
// build 命令
// ---------------------------------------------------------------------------

#[test]
fn build_valid_fixture_prints_intent() {
    let fixtures = fixtures_dir();
    let src = fixtures.join("build-src");
    let out_dir = fixtures.join("build-dist");
    let _ = std::fs::remove_dir_all(&src);
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(&src).unwrap();
    std::fs::copy(fixtures.join("valid.dejavu"), src.join("valid.dejavu")).unwrap();

    let out = run_dejavu(&[
        "build",
        "-s",
        src.to_str().unwrap(),
        "-o",
        out_dir.to_str().unwrap(),
        "--clean",
    ]);
    assert!(
        out.status.success(),
        "build should pass for valid template\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1 template(s) scanned"));
    assert!(stdout.contains("would write compiled artifacts"));
    assert!(stdout.contains("would clean output directory"));
    // 确认没有真的写文件
    assert!(!out_dir.exists() || out_dir.read_dir().unwrap().next().is_none());
}

#[test]
fn build_with_broken_fails_with_exit_1() {
    let fixtures = fixtures_dir();
    let src = fixtures.join("build-src-broken");
    let _ = std::fs::remove_dir_all(&src);
    std::fs::create_dir_all(&src).unwrap();
    std::fs::copy(fixtures.join("broken.dejavu"), src.join("broken.dejavu")).unwrap();

    let out = run_dejavu(&["build", "-s", src.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("aborted due to parse errors"));
}

// ---------------------------------------------------------------------------
// --extensions 参数覆盖
// ---------------------------------------------------------------------------

#[test]
fn extensions_flag_overrides_default() {
    let fixtures = fixtures_dir();
    let src = fixtures.join("ext-test");
    let _ = std::fs::remove_dir_all(&src);
    std::fs::create_dir_all(&src).unwrap();
    std::fs::copy(fixtures.join("valid.dejavu"), src.join("valid.dejavu")).unwrap();
    // 这个文件是 ".html" —— 默认扩展名会包含它，但我们指定只认 .dejavu
    std::fs::write(src.join("other.html"), "<%= x %>").unwrap();

    let out = run_dejavu(&[
        "check",
        "-s",
        src.to_str().unwrap(),
        "--extensions",
        ".dejavu",
    ]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1 template(s) scanned"));
}
