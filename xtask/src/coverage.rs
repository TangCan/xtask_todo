//! `coverage` subcommand - run cargo-tarpaulin per crate and report coverage.

use argh::FromArgs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "coverage")]
/// Run cargo-tarpaulin for each workspace crate and print per-crate coverage
pub struct CoverageArgs {}

/// Run coverage for a single package, streaming stdout to the terminal and returning parsed percentage.
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
            // Parse line like "|| 100.00% coverage, 61/61 lines covered" or "88.61% coverage, 179/202 lines"
            if pct.is_none() && line.contains("coverage") {
                if let Some(v) = line
                    .split_whitespace()
                    .find(|s| s.ends_with('%'))
                    .and_then(|s| s.trim_end_matches('%').parse::<f64>().ok())
                {
                    pct = Some(v);
                }
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

    println!("--- todo ---");
    let (name, pct) = run_tarpaulin("todo", &[], &[]);
    results.push((name, pct));

    println!("\n--- xtask ---");
    let (name, pct) = run_tarpaulin(
        "xtask",
        &["--exclude-files", "xtask/src/main.rs"],
        &["--test-threads=1"],
    );
    results.push((name, pct));

    println!("\n| Crate  | Coverage |");
    println!("|--------|----------|");
    for (crate_name, pct_opt) in &results {
        let cell = pct_opt
            .as_ref()
            .map_or_else(|| "N/A".to_string(), |p| format!("{p:.2}%"));
        println!("| {crate_name:<6} | {cell:<8} |");
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
