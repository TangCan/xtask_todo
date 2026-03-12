//! `coverage` subcommand - run cargo-tarpaulin per crate and report coverage.

use argh::FromArgs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "coverage")]
/// Run cargo-tarpaulin for each workspace crate and print per-crate coverage
pub struct CoverageArgs {}

/// Parse a line containing "X.XX% coverage" and return the percentage.
#[must_use]
pub fn parse_coverage_percentage(line: &str) -> Option<f64> {
    if !line.contains("coverage") {
        return None;
    }
    line.split_whitespace()
        .find(|s| s.ends_with('%'))
        .and_then(|s| s.trim_end_matches('%').parse::<f64>().ok())
}

/// Run coverage for a single package, streaming stdout to the terminal and returning parsed percentage.
#[cfg_attr(test, allow(dead_code))]
fn run_tarpaulin(package: &str, extra_args: &[&str], test_args: &[&str]) -> (String, Option<f64>) {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("tarpaulin")
        .arg("-p")
        .arg(package)
        .arg("--out")
        .arg("Stdout");
    cmd.args(extra_args);
    cmd.arg("--");
    cmd.args(test_args);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::inherit());

    let Ok(mut child) = cmd.spawn() else {
        return (package.to_string(), None);
    };

    let mut pct = None;
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            println!("{line}");
            if pct.is_none() {
                pct = parse_coverage_percentage(&line);
            }
        }
    }

    let _ = child.wait();
    (package.to_string(), pct)
}

/// Run coverage for each crate and print a summary table.
///
/// # Errors
/// Returns an error if tarpaulin fails for a crate (e.g. not installed).
pub fn cmd_coverage(_args: CoverageArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running coverage (cargo-tarpaulin) per crate...\n");

    let mut results = Vec::new();

    if std::env::var_os("XTASK_COVERAGE_TEST_FAKE").is_some() {
        results.push(("xtask-todo-lib".to_string(), Some(100.0)));
        results.push(("xtask".to_string(), Some(95.0)));
    } else if std::env::var_os("XTASK_COVERAGE_TEST_FAKE_FAIL").is_some() {
        results.push(("xtask-todo-lib".to_string(), None));
        results.push(("xtask".to_string(), None));
    } else {
        println!("--- xtask-todo-lib ---");
        let (name, pct) = run_tarpaulin("xtask-todo-lib", &[], &[]);
        results.push((name, pct));

        println!("\n--- xtask ---");
        let (name, pct) = run_tarpaulin(
            "xtask",
            &[
                "--exclude-files",
                "xtask/src/main.rs",
                "--exclude-files",
                "crates/todo/*",
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
        eprintln!("\nInstall with: cargo install cargo-tarpaulin");
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
    fn run_tarpaulin_spawn_fail_returns_none() {
        std::env::set_var("CARGO", "/nonexistent/cargo-path");
        let (name, pct) = run_tarpaulin("some-package", &[], &[]);
        std::env::remove_var("CARGO");
        assert_eq!(name, "some-package");
        assert!(pct.is_none());
    }

    /// Covers `run_tarpaulin` success path and `cmd_coverage` real branch by using a fake CARGO that echoes a coverage line.
    #[test]
    #[cfg(unix)]
    fn run_tarpaulin_fake_script_returns_pct_and_cmd_coverage_succeeds() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("xtask_cov_fake_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let script = dir.join("fake_cargo");
        let mut f = std::fs::File::create(&script).unwrap();
        f.write_all(b"#!/bin/sh\necho '|| 100.00% coverage, 61/61 lines covered'\n")
            .unwrap();
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
        let _ = std::fs::remove_dir_all(dir);
        assert!(
            out.is_ok(),
            "cmd_coverage with fake CARGO should succeed: {out:?}"
        );
    }
}
