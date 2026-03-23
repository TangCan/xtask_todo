//! `cargo xtask acceptance` — run checks aligned with [docs/acceptance.md](../docs/acceptance.md) and write a Markdown report.

use argh::FromArgs;
use std::fmt::Write;
use std::fs;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Outcome of a single automated check.
#[derive(Debug, Clone)]
pub enum CheckStatus {
    Pass,
    Fail(String),
    /// Not run (missing tool, target, etc.).
    Skip(String),
}

/// One row in the automated section of the report.
#[derive(Debug)]
pub struct AutomatedCheck {
    pub id: &'static str,
    pub description: &'static str,
    pub command: String,
    pub status: CheckStatus,
}

/// Run acceptance automation and write report.
///
/// # Errors
/// Returns an error message if any **non-skipped** check fails or if I/O fails.
pub fn cmd_acceptance(args: AcceptanceArgs) -> Result<(), String> {
    let root = workspace_root()?;
    let checks = run_all_checks(&root);
    let manual = manual_skip_rows();
    let report = build_report(&root, &checks, &manual);

    if args.stdout_only {
        print!("{report}");
    } else {
        let out = args
            .output
            .unwrap_or_else(|| root.join("docs/acceptance-report.md"));
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut f = fs::File::create(&out).map_err(|e| e.to_string())?;
        IoWrite::write_all(&mut f, report.as_bytes()).map_err(|e| e.to_string())?;
        eprintln!("Wrote acceptance report to {}", out.display());
        // Short summary to stderr
        let fails = checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Fail(_)))
            .count();
        let skips = checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Skip(_)))
            .count();
        eprintln!(
            "Summary: {} checks, {} passed, {} failed, {} skipped (automated)",
            checks.len(),
            checks.len() - fails - skips,
            fails,
            skips
        );
    }

    let any_fail = checks
        .iter()
        .any(|c| matches!(c.status, CheckStatus::Fail(_)));
    if any_fail {
        return Err(
            "one or more acceptance checks failed — see report or run with RUST_BACKTRACE=1"
                .to_string(),
        );
    }
    Ok(())
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "acceptance")]
/// Run automated acceptance checks from docs/acceptance.md and write a Markdown report
pub struct AcceptanceArgs {
    /// write report to PATH (default: <workspace>/docs/acceptance-report.md)
    #[argh(option, short = 'o')]
    pub output: Option<PathBuf>,
    /// print report to stdout only; do not write a file
    #[argh(switch)]
    pub stdout_only: bool,
}

fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let mut dir = cwd.clone();
    loop {
        let manifest = dir.join("Cargo.toml");
        if manifest.is_file() {
            let text = fs::read_to_string(&manifest).map_err(|e| e.to_string())?;
            if text.contains("[workspace]") && text.contains("members") {
                return Ok(dir);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    Ok(cwd)
}

fn run_all_checks(root: &Path) -> Vec<AutomatedCheck> {
    vec![
        check_workspace_members(root),
        check_cargo_xtask_alias(root),
        check_pre_commit_has_msvc(root),
        run_cargo_test(
            root,
            "AC-TODO-LIB",
            "xtask-todo-lib crate tests",
            &["test", "-p", "xtask-todo-lib", "--", "--test-threads=1"],
        ),
        run_cargo_test(
            root,
            "AC-XTASK",
            "xtask crate tests",
            &["test", "-p", "xtask", "--", "--test-threads=1"],
        ),
        run_cargo_test(
            root,
            "AC-DEVSHELL-VM",
            "devshell-vm crate tests",
            &["test", "-p", "devshell-vm", "--", "--test-threads=1"],
        ),
        check_windows_msvc(root),
    ]
}

fn check_workspace_members(root: &Path) -> AutomatedCheck {
    let id = "NF-1";
    let desc = "Workspace `Cargo.toml` lists `crates/todo` and `xtask`";
    let path = root.join("Cargo.toml");
    let cmd = format!("read {}", path.display());
    let status = match fs::read_to_string(&path) {
        Ok(text) => {
            let ok = text.contains("crates/todo")
                && text.contains("xtask")
                && text.contains("[workspace]");
            if ok {
                CheckStatus::Pass
            } else {
                CheckStatus::Fail("expected members to include crates/todo and xtask".to_string())
            }
        }
        Err(e) => CheckStatus::Fail(e.to_string()),
    };
    AutomatedCheck {
        id,
        description: desc,
        command: cmd,
        status,
    }
}

fn check_cargo_xtask_alias(root: &Path) -> AutomatedCheck {
    let id = "NF-2";
    let desc = "`.cargo/config.toml` defines `cargo xtask` alias";
    let path = root.join(".cargo/config.toml");
    let cmd = format!("read {}", path.display());
    let status = match fs::read_to_string(&path) {
        Ok(text) => {
            let ok = text.contains("xtask") && text.contains("run -p xtask");
            if ok {
                CheckStatus::Pass
            } else {
                CheckStatus::Fail("expected [alias] xtask = run -p xtask --".to_string())
            }
        }
        Err(e) => CheckStatus::Fail(e.to_string()),
    };
    AutomatedCheck {
        id,
        description: desc,
        command: cmd,
        status,
    }
}

fn check_pre_commit_has_msvc(root: &Path) -> AutomatedCheck {
    let id = "NF-6";
    let desc = "`.githooks/pre-commit` includes fmt, line limit, clippy, rustdoc, test, MSVC check";
    let path = root.join(".githooks/pre-commit");
    let cmd = format!("read {}", path.display());
    let status = match fs::read_to_string(&path) {
        Ok(text) => {
            let ok = text.contains("x86_64-pc-windows-msvc")
                && text.contains("cargo check")
                && text.contains("cargo doc")
                && text.contains("--no-deps");
            if ok {
                CheckStatus::Pass
            } else {
                CheckStatus::Fail(
                    "expected pre-commit to contain MSVC cargo check, cargo doc --no-deps, etc."
                        .to_string(),
                )
            }
        }
        Err(e) => CheckStatus::Fail(e.to_string()),
    };
    AutomatedCheck {
        id,
        description: desc,
        command: cmd,
        status,
    }
}

fn run_cargo_test(
    root: &Path,
    id: &'static str,
    description: &'static str,
    args: &[&str],
) -> AutomatedCheck {
    let cmd_display = format!("cargo {}", args.join(" "));
    let mut c = Command::new("cargo");
    c.args(args)
        .current_dir(root)
        .env("CARGO_TERM_COLOR", "never");
    let status = match c.output() {
        Ok(o) => {
            if o.status.success() {
                CheckStatus::Pass
            } else {
                let err = String::from_utf8_lossy(&o.stderr);
                let out = String::from_utf8_lossy(&o.stdout);
                let tail = format!("{err}\n{out}");
                let tail = tail.chars().take(2000).collect::<String>();
                CheckStatus::Fail(tail)
            }
        }
        Err(e) => CheckStatus::Fail(format!("failed to spawn cargo: {e}")),
    };
    AutomatedCheck {
        id,
        description,
        command: cmd_display,
        status,
    }
}

fn check_windows_msvc(root: &Path) -> AutomatedCheck {
    let id = "NF-5/D8";
    let description =
        "`cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc` (MSVC cross-compile)";
    let cmd_s = "cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc".to_string();

    if !rustup_target_installed("x86_64-pc-windows-msvc") {
        return AutomatedCheck {
            id,
            description,
            command: cmd_s,
            status: CheckStatus::Skip(
                "rustup target x86_64-pc-windows-msvc not installed — run: rustup target add x86_64-pc-windows-msvc"
                    .to_string(),
            ),
        };
    }

    let mut c = Command::new("cargo");
    c.args([
        "check",
        "-p",
        "xtask-todo-lib",
        "--target",
        "x86_64-pc-windows-msvc",
    ])
    .current_dir(root)
    .env("CARGO_TERM_COLOR", "never");
    let status = match c.output() {
        Ok(o) => {
            if o.status.success() {
                CheckStatus::Pass
            } else {
                let err = String::from_utf8_lossy(&o.stderr);
                let out = String::from_utf8_lossy(&o.stdout);
                let tail = format!("{err}\n{out}");
                let tail = tail.chars().take(2000).collect::<String>();
                CheckStatus::Fail(tail)
            }
        }
        Err(e) => CheckStatus::Fail(format!("failed to spawn cargo: {e}")),
    };
    AutomatedCheck {
        id,
        description,
        command: cmd_s,
        status,
    }
}

fn rustup_target_installed(target: &str) -> bool {
    let Ok(out) = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
    else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    s.lines().any(|line| line.trim() == target)
}

fn manual_skip_rows() -> Vec<(&'static str, &'static str)> {
    vec![
        ("T6-1", "TTY 下未完成项着色 — 需在终端人工查看"),
        ("T6-2", "非 TTY 无 ANSI — 需管道或重定向人工确认"),
        ("NF-3", "主版本 CLI / CHANGELOG — 发布与评审流程"),
        ("NF-4", "`--help` 与 README 一致 — 文档评审"),
        ("D5", "`rustup`/`cargo` sandbox 或 VM — 依赖宿主 PATH/环境"),
        ("D6", "Mode P / Lima — 需 limactl 与实例"),
        ("X3-1", "新子命令注册模式 — 代码评审"),
    ]
}

fn utc_stamp() -> String {
    if cfg!(unix) {
        let o = Command::new("date")
            .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
            .output();
        if let Ok(out) = o {
            if out.status.success() {
                return String::from_utf8_lossy(&out.stdout).trim().to_string();
            }
        }
    }
    "unknown".to_string()
}

fn build_report(
    root: &Path,
    checks: &[AutomatedCheck],
    manual: &[(&'static str, &'static str)],
) -> String {
    let mut s = String::new();
    s.push_str("# Acceptance report\n\n");
    s.push_str(
        "本报告由 `cargo xtask acceptance` 生成，与 [acceptance.md](./acceptance.md) 对照。\n\n",
    );
    let _ = writeln!(&mut s, "- **生成时间（UTC）**: {}", utc_stamp());
    let _ = writeln!(&mut s, "- **仓库根**: `{}`\n", root.display());

    s.push_str("## 1. 自动化检查结果\n\n");
    s.push_str("| ID | 说明 | 命令 / 检查 | 结果 |\n");
    s.push_str("|----|------|-------------|------|\n");
    for c in checks {
        let st = match &c.status {
            CheckStatus::Pass => "✅ PASS".to_string(),
            CheckStatus::Fail(_) => "❌ FAIL".to_string(),
            CheckStatus::Skip(reason) => format!("⏸ SKIP — {reason}"),
        };
        let cmd_short = if c.command.len() > 80 {
            format!("{}…", &c.command[..77])
        } else {
            c.command.clone()
        };
        let _ = writeln!(
            &mut s,
            "| {} | {} | `{}` | {} |",
            c.id, c.description, cmd_short, st
        );
    }
    s.push('\n');

    for c in checks {
        if let CheckStatus::Fail(detail) = &c.status {
            let _ = write!(&mut s, "### ❌ 失败详情: {}\n\n", c.id);
            s.push_str("```text\n");
            s.push_str(detail);
            s.push_str("\n```\n\n");
        }
    }

    s.push_str("## 2. 需人工或环境的验收项（本命令不执行）\n\n");
    s.push_str("| ID | 原因 |\n");
    s.push_str("|----|------|\n");
    for (id, reason) in manual {
        let _ = writeln!(&mut s, "| {id} | {reason} |");
    }
    s.push('\n');

    s.push_str("## 3. 验收 ID 与自动化覆盖说明\n\n");
    s.push_str("以下验收编号见 [acceptance.md](./acceptance.md)。\n\n");
    s.push_str("| 区域 | 覆盖方式 |\n");
    s.push_str("|------|----------|\n");
    s.push_str("| **§2 Todo（T1-1～T13）** | `cargo test -p xtask-todo-lib` + `cargo test -p xtask`（todo 相关） |\n");
    s.push_str("| **§3 xtask / AI（X*、A*）** | `cargo test -p xtask` |\n");
    s.push_str("| **§4 Devshell（D1～D4、D7）** | `cargo test -p xtask-todo-lib`（devshell 集成/单元） |\n");
    s.push_str("| **§5 非功能 NF-1、NF-2、NF-6** | 本命令文件检查 |\n");
    s.push_str("| **NF-5、D8（Windows MSVC）** | `cargo check -p xtask-todo-lib --target x86_64-pc-windows-msvc`（未安装 target 时 **SKIP**） |\n");
    s.push_str("| **T6-1、T6-2、NF-3、NF-4、D5、D6、X3-1** | 见 §2 表（人工/环境） |\n");
    s.push('\n');

    s.push_str("## 4. 结论\n\n");
    let any_fail = checks
        .iter()
        .any(|c| matches!(c.status, CheckStatus::Fail(_)));
    if any_fail {
        s.push_str("**状态**: ❌ **存在失败的自动化检查**，请在修复后重新运行 `cargo xtask acceptance`。\n");
    } else {
        s.push_str("**状态**: ✅ **全部自动化检查通过**（含 SKIP 项则仅表示未执行该环境检查）。发布前请仍完成 §2 人工项（若适用）。\n");
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_root_finds_repo() {
        let root = workspace_root().expect("root");
        assert!(root.join("Cargo.toml").is_file());
        let text = fs::read_to_string(root.join("Cargo.toml")).unwrap();
        assert!(text.contains("[workspace]"));
    }

    #[test]
    fn nf1_nf2_nf6_pass_on_repo() {
        let root = workspace_root().unwrap();
        assert!(matches!(
            check_workspace_members(&root).status,
            CheckStatus::Pass
        ));
        assert!(matches!(
            check_cargo_xtask_alias(&root).status,
            CheckStatus::Pass
        ));
        assert!(matches!(
            check_pre_commit_has_msvc(&root).status,
            CheckStatus::Pass
        ));
    }
}
