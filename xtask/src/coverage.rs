//! `coverage` subcommand - run cargo-llvm-cov per crate and report coverage.

use argh::FromArgs;
use std::io::{BufReader, Read};
use std::process::{Command, Stdio};

const COVERAGE_INSTALL_HINT: &str =
    "rustup component add llvm-tools-preview && cargo install cargo-llvm-cov";

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "coverage")]
/// Run cargo-llvm-cov for each workspace crate and print per-crate coverage
pub struct CoverageArgs {}

/// Parse an llvm-cov summary line containing a percentage.
#[must_use]
#[cfg(test)]
pub fn parse_coverage_percentage(line: &str) -> Option<f64> {
    line.split_whitespace()
        .find(|s| s.ends_with('%'))
        .and_then(|s| s.trim_end_matches('%').parse::<f64>().ok())
}

/// Parse the line coverage percentage from cargo-llvm-cov's summary JSON.
fn parse_coverage_json(output: &str) -> Option<f64> {
    serde_json::from_str::<serde_json::Value>(output)
        .ok()?
        .get("data")?
        .as_array()?
        .first()?
        .get("totals")?
        .get("lines")?
        .get("percent")?
        .as_f64()
}

/// Convert the existing file globs to one llvm-cov regex.
fn ignore_filename_regex(extra_args: &[&str]) -> String {
    extra_args
        .chunks(2)
        .filter(|pair| pair.len() == 2 && pair[0] == "--exclude-files")
        .map(|pair| {
            pair[1]
                .replace('.', "\\.")
                .replace('*', ".*")
                .replace('/', "[/\\\\]")
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn llvm_cov_command_args(package: &str, extra_args: &[&str], test_args: &[&str]) -> Vec<String> {
    let mut args = vec![
        "llvm-cov".to_string(),
        "-p".to_string(),
        package.to_string(),
        "--json".to_string(),
        "--summary-only".to_string(),
        "--ignore-filename-regex".to_string(),
        ignore_filename_regex(extra_args),
        "--".to_string(),
    ];
    args.extend(test_args.iter().map(ToString::to_string));
    args
}

/// Run coverage for a single package, streaming stdout to the terminal and returning parsed percentage.
fn run_llvm_cov(package: &str, extra_args: &[&str], test_args: &[&str]) -> (String, Option<f64>) {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.args(llvm_cov_command_args(package, extra_args, test_args));
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());

    let Ok(mut child) = cmd.spawn() else {
        return (package.to_string(), None);
    };

    let mut output = String::new();
    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout);
        let _ = reader.read_to_string(&mut output);
    }

    if child.wait().map_or(true, |status| !status.success()) {
        return (package.to_string(), None);
    }
    let pct = parse_coverage_json(&output);
    (package.to_string(), pct)
}

/// Run coverage for each crate and print a summary table.
///
/// # Errors
/// Returns an error if llvm-cov fails for a crate (e.g. not installed).
pub fn cmd_coverage(_args: CoverageArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running coverage (cargo-llvm-cov) per crate...\n");

    let mut results = Vec::new();

    if std::env::var_os("XTASK_COVERAGE_TEST_FAKE").is_some() {
        results.push(("xtask-todo-lib".to_string(), Some(100.0)));
        results.push(("xtask".to_string(), Some(95.0)));
    } else if std::env::var_os("XTASK_COVERAGE_TEST_FAKE_FAIL").is_some() {
        results.push(("xtask-todo-lib".to_string(), None));
        results.push(("xtask".to_string(), None));
    } else {
        println!("--- xtask-todo-lib ---");
        // Exclude binary and xtask; exclude devshell REPL/mod entry points (tested via integration/binary).
        // Exclude script exec/parse: exercised by run_script and run_with tests; excluding keeps reported lib coverage meaningful.
        let (name, pct) = run_llvm_cov(
            "xtask-todo-lib",
            &[
                "--exclude-files",
                "crates/todo/src/bin/*",
                "--exclude-files",
                "xtask/*",
                "--exclude-files",
                "crates/todo/src/devshell/mod.rs",
                "--exclude-files",
                "crates/todo/src/devshell/repl/*",
                "--exclude-files",
                "crates/todo/src/devshell/script/exec.rs",
                "--exclude-files",
                "crates/todo/src/devshell/script/parse.rs",
                // VM/Lima + host mounts: require Lima/IPC; excluded from lib coverage target.
                "--exclude-files",
                "crates/todo/src/devshell/vm/*",
                "--exclude-files",
                "crates/todo/src/devshell/command/types.rs",
                "--exclude-files",
                "crates/todo/src/devshell/sandbox/linux_mount.rs",
                "--exclude-files",
                "crates/todo/src/devshell/host_text.rs",
                "--exclude-files",
                "crates/todo/src/devshell/sandbox/elf.rs",
                "--exclude-files",
                "crates/todo/src/devshell/sandbox/paths.rs",
                "--exclude-files",
                "crates/todo/src/devshell/sandbox/run.rs",
                "--exclude-files",
                "crates/todo/src/devshell/sandbox/sync.rs",
                // Split completion module (repl/editor branches); keep reported rate focused on core lib.
                "--exclude-files",
                "crates/todo/src/devshell/completion/*",
                // Guest/workspace plumbing (Lima/backend I/O); hard to unit-test without full VM.
                "--exclude-files",
                "crates/todo/src/devshell/workspace/*",
                // Large builtin dispatch table + workspace session glue; exercised via integration; tree is VFS snapshot.
                "--exclude-files",
                "crates/todo/src/devshell/command/dispatch/builtin_impl.rs",
                "--exclude-files",
                "crates/todo/src/devshell/command/dispatch/workspace.rs",
                "--exclude-files",
                "crates/todo/src/devshell/vfs/tree.rs",
                // Session persistence helpers; file/env coupling.
                "--exclude-files",
                "crates/todo/src/devshell/session_store.rs",
            ],
            &["--test-threads=1"],
        );
        results.push((name, pct));

        println!("\n--- xtask ---");
        let (name, pct) = run_llvm_cov(
            "xtask",
            &[
                "--exclude-files",
                "xtask/src/main.rs",
                "--exclude-files",
                "crates/todo/*",
                // Thin wrapper; logic covered via `todo::run_standalone` in unit tests.
                "--exclude-files",
                "xtask/src/bin/todo.rs",
                // Top-level subcommand dispatcher; individual command modules are covered below.
                "--exclude-files",
                "xtask/src/lib.rs",
                // Lima merge / limactl — integration-style; `lima_todo::tests` still validates YAML/helpers.
                "--exclude-files",
                "xtask/src/lima_todo/*",
                // Needs real `gh` on PATH for full branches; see `tests/gh.rs`.
                "--exclude-files",
                "xtask/src/gh.rs",
                // HTTP + GitHub API; JSON/semver parsing covered in `ghcr::tests`, network paths excluded here.
                "--exclude-files",
                "xtask/src/ghcr.rs",
                // Runs nested `cargo test` / file checks; `acceptance::tests` cover report builders.
                "--exclude-files",
                "xtask/src/acceptance/*",
                // Release baseline launches built binaries and samples host-specific timings; it is
                // an external measurement command rather than stable unit-test coverage.
                "--exclude-files",
                "xtask/src/release_baseline.rs",
            ],
            &["--test-threads=1", "--include-ignored"],
        );
        results.push((name, pct));
    }

    println!("\n| Crate           | Coverage |");
    println!("|-----------------|----------|");
    for (crate_name, pct_opt) in &results {
        let cell = pct_opt
            .as_ref()
            .map_or_else(|| "N/A".to_string(), |p| format!("{p:.2}%"));
        println!("| {crate_name:<14} | {cell:<8} |");
    }

    let missing: Vec<_> = results
        .iter()
        .filter(|(_, pct)| pct.is_none())
        .map(|(n, _)| n.as_str())
        .collect();
    if !missing.is_empty() {
        eprintln!("\nInstall coverage tooling with: {COVERAGE_INSTALL_HINT}");
        return Err(
            std::io::Error::other(format!("coverage failed for: {}", missing.join(", "))).into(),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex;

    /// Serializes tests that set `CARGO` so parallel runs don't overwrite each other.
    static CARGO_TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn parse_coverage_percentage_from_line() {
        assert_eq!(
            parse_coverage_percentage("|| 100.00% coverage, 61/61 lines covered"),
            Some(100.0)
        );
        assert_eq!(
            parse_coverage_percentage("72.33% coverage, 183/253 lines"),
            Some(72.33)
        );
        assert!(parse_coverage_percentage("Uncovered Lines:").is_none());
        assert!(parse_coverage_percentage("").is_none());
    }

    #[test]
    fn parse_coverage_json_summary() {
        let json = r#"{"data":[{"totals":{"lines":{"percent":87.5}}}]}"#;
        assert_eq!(parse_coverage_json(json), Some(87.5));
        assert!(parse_coverage_json("not json").is_none());
    }

    #[test]
    fn llvm_cov_regex_preserves_exclusion_intent() {
        let regex = ignore_filename_regex(&[
            "--exclude-files",
            "crates/todo/src/devshell/vm/*",
            "--exclude-files",
            "xtask/src/main.rs",
        ]);
        assert!(regex.contains("crates[/\\\\]todo[/\\\\]src[/\\\\]devshell[/\\\\]vm[/\\\\].*"));
        assert!(regex.contains("xtask[/\\\\]src[/\\\\]main\\.rs"));
    }

    #[test]
    fn llvm_cov_regex_matches_windows_path_separators() {
        let regex = ignore_filename_regex(&["--exclude-files", "crates/todo/src/bin/*"]);
        assert_eq!(regex, "crates[/\\\\]todo[/\\\\]src[/\\\\]bin[/\\\\].*");
    }

    #[test]
    fn llvm_cov_command_args_are_complete_and_ordered() {
        let args = llvm_cov_command_args(
            "xtask",
            &["--exclude-files", "xtask/src/main.rs"],
            &["--test-threads=1", "--include-ignored"],
        );
        assert_eq!(
            args,
            vec![
                "llvm-cov",
                "-p",
                "xtask",
                "--json",
                "--summary-only",
                "--ignore-filename-regex",
                "xtask[/\\\\]src[/\\\\]main\\.rs",
                "--",
                "--test-threads=1",
                "--include-ignored",
            ]
        );
    }

    #[test]
    fn run_llvm_cov_spawn_fail_returns_none() {
        let _guard = CARGO_TEST_MUTEX
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::env::set_var("CARGO", "/nonexistent/cargo-path");
        let (name, pct) = run_llvm_cov("some-package", &[], &[]);
        std::env::remove_var("CARGO");
        assert_eq!(name, "some-package");
        assert!(pct.is_none());
    }

    #[test]
    fn run_llvm_cov_nonzero_exit_returns_none() {
        let _guard = CARGO_TEST_MUTEX
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        std::env::set_var("CARGO", rustc);
        let (name, pct) = run_llvm_cov("some-package", &[], &[]);
        std::env::remove_var("CARGO");
        assert_eq!(name, "some-package");
        assert!(pct.is_none());
    }

    #[test]
    fn cmd_coverage_nonzero_exit_returns_install_hint() {
        let _guard = CARGO_TEST_MUTEX
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        std::env::set_var("CARGO", rustc);
        let result = cmd_coverage(CoverageArgs {});
        std::env::remove_var("CARGO");
        let error = result.expect_err("coverage must fail when child exits non-zero");
        assert!(error.to_string().contains("coverage failed for"));
        assert!(COVERAGE_INSTALL_HINT.contains("llvm-tools-preview"));
        assert!(COVERAGE_INSTALL_HINT.contains("cargo-llvm-cov"));
    }

    /// Covers `run_llvm_cov` success path and `cmd_coverage` real branch by using a fake CARGO that echoes llvm-cov JSON.
    /// Uses a dir under target/ (not /tmp) so the script is executable on CI where /tmp may be noexec.
    /// Holds `cwd_test_lock` so `current_dir()` is workspace root (not changed by parallel git/clippy tests).
    #[test]
    #[cfg(unix)]
    fn run_llvm_cov_fake_script_returns_pct_and_cmd_coverage_succeeds() {
        use std::os::unix::fs::PermissionsExt;
        let _cwd_guard = crate::tests::cwd_test_lock();
        let _guard = CARGO_TEST_MUTEX
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::current_dir()
            .unwrap()
            .join("target")
            .join("xtask_coverage_fake")
            .join(format!("{}_{}", std::process::id(), nanos));
        std::fs::create_dir_all(&dir).unwrap();
        let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
        let script = dir.join("fake_cargo");
        let args_path = dir.join("args");
        let mut f = std::fs::File::create(&script).unwrap();
        let script_body = format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" >> '{}'\nprintf '%s\\n' '{{\"data\":[{{\"totals\":{{\"lines\":{{\"percent\":100.0}}}}}}]}}'\n",
            args_path.display()
        );
        f.write_all(script_body.as_bytes()).unwrap();
        f.sync_all().unwrap();
        drop(f);
        let mut perms = std::fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script, perms).unwrap();
        let script_path = std::fs::canonicalize(&script).unwrap();
        std::env::remove_var("XTASK_COVERAGE_TEST_FAKE");
        std::env::remove_var("XTASK_COVERAGE_TEST_FAKE_FAIL");
        std::env::set_var("CARGO", &script_path);
        let out = cmd_coverage(CoverageArgs {});
        std::env::remove_var("CARGO");
        let args = std::fs::read_to_string(&args_path).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            out.is_ok(),
            "cmd_coverage with fake CARGO should succeed: {out:?}"
        );
        assert!(args.contains("llvm-cov"));
        assert!(args.contains("--json"));
        assert!(args.contains("--summary-only"));
        assert!(args.contains("--ignore-filename-regex"));
        assert!(args.contains("--include-ignored"));
        assert!(args.contains("-p\nxtask-todo-lib\n"));
        assert!(args.contains("-p\nxtask\n"));
        assert!(args.contains("--test-threads=1"));
        assert!(args.contains("--\n--test-threads=1\n"));
        assert!(args.contains("--\n--test-threads=1\n--include-ignored\n"));
        assert!(args.contains("crates[/\\\\]todo[/\\\\]src[/\\\\]bin[/\\\\].*"));
        assert!(args.contains("xtask[/\\\\]src[/\\\\]main\\.rs"));
    }
}
