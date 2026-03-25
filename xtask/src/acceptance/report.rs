//! Markdown report body and manual-only rows.

use std::fmt::Write;
use std::path::Path;
use std::process::Command;

use super::{AutomatedCheck, CheckStatus};

pub(super) fn manual_skip_rows() -> Vec<(&'static str, &'static str)> {
    vec![
        ("T6-1", "TTY 下未完成项着色 — 需在终端人工查看"),
        ("T6-2", "非 TTY 无 ANSI — 需管道或重定向人工确认"),
        ("NF-3", "主版本 CLI / CHANGELOG — 发布与评审流程"),
        ("NF-4", "`--help` 与 README 一致 — 文档评审"),
        ("D5", "`rustup`/`cargo` sandbox 或 VM — 依赖宿主 PATH/环境"),
        ("D6", "Mode P / Lima — 需 limactl 与实例"),
        ("D9", "Windows β + Podman 全链路 — 需专用环境手工验证"),
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

pub(super) fn build_report(
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
    s.push_str("| **T6-1、T6-2、NF-3、NF-4、D5、D6、D9、X3-1** | 见 §2 表（人工/环境） |\n");
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
