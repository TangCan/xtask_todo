//! Individual automated checks (`NF-*`, `cargo test`, MSVC cross-compile).

use std::path::Path;
use std::process::Command;

use super::{AutomatedCheck, CheckStatus};

pub(super) fn run_all_checks(root: &Path) -> Vec<AutomatedCheck> {
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

pub(super) fn check_workspace_members(root: &Path) -> AutomatedCheck {
    let id = "NF-1";
    let desc = "Workspace `Cargo.toml` lists `crates/todo` and `xtask`";
    let path = root.join("Cargo.toml");
    let cmd = format!("read {}", path.display());
    let status = match std::fs::read_to_string(&path) {
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

pub(super) fn check_cargo_xtask_alias(root: &Path) -> AutomatedCheck {
    let id = "NF-2";
    let desc = "`.cargo/config.toml` defines `cargo xtask` alias";
    let path = root.join(".cargo/config.toml");
    let cmd = format!("read {}", path.display());
    let status = match std::fs::read_to_string(&path) {
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

pub(super) fn check_pre_commit_has_msvc(root: &Path) -> AutomatedCheck {
    let id = "NF-6";
    let desc = "`.githooks/pre-commit` includes fmt, line limit, clippy, rustdoc, test, MSVC check";
    let path = root.join(".githooks/pre-commit");
    let cmd = format!("read {}", path.display());
    let status = match std::fs::read_to_string(&path) {
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
