//! `coverage` subcommand - run cargo-tarpaulin per crate and report coverage.

use argh::FromArgs;
use std::process::Command;

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "coverage")]
/// Run cargo-tarpaulin for each workspace crate and print per-crate coverage
pub struct CoverageArgs {}

/// Run coverage for a single package, returning (`package_name`, `percentage_string` or error message).
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

    let Ok(output) = cmd.output() else {
        return (package.to_string(), None);
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    // Parse line like "|| 100.00% coverage, 61/61 lines covered" or "88.61% coverage, 179/202 lines"
    for line in combined.lines() {
        if line.contains("coverage") {
            if let Some(pct) = line
                .split_whitespace()
                .find(|s| s.ends_with('%'))
                .and_then(|s| s.trim_end_matches('%').parse::<f64>().ok())
            {
                return (package.to_string(), Some(pct));
            }
        }
    }

    (package.to_string(), None)
}

/// Run coverage for each crate and print a summary table.
///
/// # Errors
/// Returns an error if tarpaulin fails for a crate (e.g. not installed).
pub fn cmd_coverage(_args: CoverageArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running coverage (cargo-tarpaulin) per crate...\n");

    let mut results = Vec::new();

    let (name, pct) = run_tarpaulin("todo", &[], &[]);
    results.push((name, pct));

    let (name, pct) = run_tarpaulin(
        "xtask",
        &["--exclude-files", "xtask/src/main.rs"],
        &["--test-threads=1"],
    );
    results.push((name, pct));

    println!("| Crate  | Coverage |");
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
