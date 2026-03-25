use std::process::Command;

fn xtask_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
}

#[test]
fn xtask_help_lists_documented_subcommands() {
    let out = xtask_bin().arg("--help").output().unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let s = String::from_utf8_lossy(&out.stdout);
    for cmd in [
        "acceptance",
        "run",
        "clean",
        "clippy",
        "coverage",
        "fmt",
        "gh",
        "ghcr",
        "git",
        "publish",
        "lima-todo",
        "todo",
    ] {
        assert!(s.contains(cmd), "top help should contain `{cmd}`: {s}");
    }
}

#[test]
fn xtask_subcommand_help_smoke() {
    for sub in [
        "acceptance",
        "run",
        "clean",
        "clippy",
        "coverage",
        "fmt",
        "gh",
        "ghcr",
        "git",
        "publish",
        "lima-todo",
        "todo",
    ] {
        let out = xtask_bin().arg(sub).arg("--help").output().unwrap();
        assert!(
            out.status.success(),
            "{sub} --help failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
