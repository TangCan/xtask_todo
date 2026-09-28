//! `release-baseline` — measure release binary size and cold startup time.

use argh::FromArgs;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const SCHEMA_VERSION: u32 = 1;

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "release-baseline")]
/// Build release binaries and record size and cold-start measurements
pub struct ReleaseBaselineArgs {
    /// output JSON path (default: docs/benchmarks/release-baseline.json)
    #[argh(option, short = 'o')]
    pub output: Option<PathBuf>,
    /// number of independent cold-start runs per binary
    #[argh(option, default = "default_runs()")]
    pub runs: u32,
    /// cargo profile to build (default: release)
    #[argh(option, default = "default_profile()")]
    pub profile: String,
}

const fn default_runs() -> u32 {
    5
}

fn default_profile() -> String {
    "release".to_string()
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct BinaryTarget {
    name: String,
    package: String,
}

#[derive(Debug, Serialize)]
struct Baseline {
    schema_version: u32,
    measured_at_unix_seconds_utc: u64,
    git_commit: String,
    git_worktree_dirty: bool,
    rustc: String,
    cargo: String,
    host: String,
    target: String,
    profile: String,
    build_arguments: Vec<String>,
    runs: u32,
    targets: Vec<BinaryMeasurement>,
}

#[derive(Debug, Serialize)]
struct BinaryMeasurement {
    name: String,
    package: String,
    path: String,
    size_bytes: u64,
    cold_start_micros: ColdStart,
}

#[derive(Debug, Serialize)]
struct ColdStart {
    samples: Vec<u128>,
    min: u128,
    p50: u128,
    p95: u128,
}

#[derive(Debug, serde::Deserialize)]
struct Metadata {
    target_directory: String,
    packages: Vec<Package>,
}

#[derive(Debug, serde::Deserialize)]
struct Package {
    name: String,
    targets: Vec<MetadataTarget>,
}

#[derive(Debug, serde::Deserialize)]
struct MetadataTarget {
    name: String,
    kind: Vec<String>,
}

/// Run the release baseline command.
pub fn cmd_release_baseline(args: ReleaseBaselineArgs) -> Result<(), String> {
    if args.runs == 0 {
        return Err("--runs must be greater than zero".to_string());
    }

    let root = std::env::current_dir().map_err(|e| format!("current directory: {e}"))?;
    let output = args
        .output
        .unwrap_or_else(|| root.join("docs/benchmarks/release-baseline.json"));
    if output.exists() {
        return Err(format!(
            "output already exists: {}; choose a new path to avoid overwriting a baseline",
            output.display()
        ));
    }

    let metadata = cargo_metadata(&root)?;
    let targets = discover_binary_targets(&metadata);
    if targets.is_empty() {
        return Err("cargo metadata found no binary targets".to_string());
    }

    let build_arguments = build_release(&root, &args.profile)?;
    let release_dir = PathBuf::from(&metadata.target_directory).join(&args.profile);
    let measurement_dir =
        std::env::temp_dir().join(format!("xtask-release-baseline-{}", std::process::id()));
    fs::create_dir_all(&measurement_dir).map_err(|e| {
        format!(
            "create measurement directory {}: {e}",
            measurement_dir.display()
        )
    })?;
    let mut measurements = Vec::with_capacity(targets.len());
    for target in targets {
        let path = binary_path(&release_dir, &target.name);
        let size_bytes = fs::metadata(&path)
            .map_err(|e| {
                format!(
                    "target '{}' binary missing at {}: {e}",
                    target.name,
                    path.display()
                )
            })?
            .len();
        let cold_start_micros =
            measure_cold_start(&path, args.runs, &target.name, &measurement_dir)?;
        measurements.push(BinaryMeasurement {
            name: target.name,
            package: target.package,
            path: path.display().to_string(),
            size_bytes,
            cold_start_micros,
        });
    }

    let baseline = Baseline {
        schema_version: SCHEMA_VERSION,
        measured_at_unix_seconds_utc: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("system clock before Unix epoch: {e}"))?
            .as_secs(),
        git_commit: command_stdout(&root, "git", &["rev-parse", "HEAD"])?,
        git_worktree_dirty: git_worktree_dirty(&root)?,
        rustc: command_stdout(&root, "rustc", &["-Vv"])?,
        cargo: command_stdout(&root, "cargo", &["-V"])?,
        host: std::env::var("HOST")
            .unwrap_or_else(|_| format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)),
        target: rustc_host(&root)?,
        profile: args.profile,
        build_arguments,
        runs: args.runs,
        targets: measurements,
    };
    let _ = fs::remove_dir_all(&measurement_dir);
    write_json_atomically(&output, &baseline)?;
    println!("wrote release baseline to {}", output.display());
    Ok(())
}

fn cargo_metadata(root: &Path) -> Result<Metadata, String> {
    let output = Command::new("cargo")
        .current_dir(root)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|e| format!("cargo metadata: {e}"))?;
    if !output.status.success() {
        return Err(format!("cargo metadata failed: {}", stderr(&output.stderr)));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| format!("cargo metadata JSON: {e}"))
}

fn discover_binary_targets(metadata: &Metadata) -> Vec<BinaryTarget> {
    let mut targets = metadata
        .packages
        .iter()
        .flat_map(|package| {
            package
                .targets
                .iter()
                .filter(|target| target.kind.iter().any(|kind| kind == "bin"))
                .map(|target| BinaryTarget {
                    name: target.name.clone(),
                    package: package.name.clone(),
                })
        })
        .collect::<Vec<_>>();
    targets.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.package.cmp(&b.package)));
    targets
}

fn build_release(root: &Path, profile: &str) -> Result<Vec<String>, String> {
    let mut arguments = vec!["build".to_string(), "--workspace".to_string()];
    if profile == "release" {
        arguments.push("--release".to_string());
    } else {
        arguments.extend(["--profile".to_string(), profile.to_string()]);
    }
    let output = Command::new("cargo")
        .current_dir(root)
        .args(&arguments)
        .output()
        .map_err(|e| format!("cargo build failed to start: {e}"))?;
    if !output.status.success() {
        return Err(format!("cargo build failed: {}", stderr(&output.stderr)));
    }
    Ok(arguments)
}

fn binary_path(release_dir: &Path, name: &str) -> PathBuf {
    #[cfg(windows)]
    {
        release_dir.join(format!("{name}.exe"))
    }
    #[cfg(not(windows))]
    {
        release_dir.join(name)
    }
}

fn measure_cold_start(
    path: &Path,
    runs: u32,
    name: &str,
    measurement_dir: &Path,
) -> Result<ColdStart, String> {
    let mut samples = Vec::with_capacity(runs as usize);
    for _ in 0..runs {
        let start = Instant::now();
        let output = Command::new(path)
            .arg("--help")
            .current_dir(measurement_dir)
            .env_remove("DEVSHELL_WORKSPACE_ROOT")
            .output()
            .map_err(|e| format!("target '{name}' failed to start: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "target '{name}' --help exited with {}: {}",
                output.status,
                stderr(&output.stderr)
            ));
        }
        samples.push(start.elapsed().as_micros());
    }
    let mut sorted = samples.clone();
    sorted.sort_unstable();
    Ok(ColdStart {
        min: sorted[0],
        p50: percentile(&sorted, 50),
        p95: percentile(&sorted, 95),
        samples,
    })
}

const fn percentile(sorted: &[u128], percentile: usize) -> u128 {
    let index = (sorted.len() - 1) * percentile / 100;
    sorted[index]
}

fn command_stdout(root: &Path, command: &str, arguments: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .current_dir(root)
        .args(arguments)
        .output()
        .map_err(|e| format!("{command} {}: {e}", arguments.join(" ")))?;
    if !output.status.success() {
        return Err(format!("{command} failed: {}", stderr(&output.stderr)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn rustc_host(root: &Path) -> Result<String, String> {
    command_stdout(root, "rustc", &["-Vv"])?
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(ToString::to_string))
        .ok_or_else(|| "rustc -Vv did not report a host target".to_string())
}

fn git_worktree_dirty(root: &Path) -> Result<bool, String> {
    let unstaged = Command::new("git")
        .current_dir(root)
        .args(["diff", "--quiet", "HEAD", "--"])
        .status()
        .map_err(|e| format!("git diff: {e}"))?;
    if !unstaged.success() {
        return Ok(true);
    }
    let staged = Command::new("git")
        .current_dir(root)
        .args(["diff", "--cached", "--quiet"])
        .status()
        .map_err(|e| format!("git diff --cached: {e}"))?;
    Ok(!staged.success())
}

fn write_json_atomically<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let temporary = path.with_extension(format!("json.tmp-{}", std::process::id()));
    let contents =
        serde_json::to_string_pretty(value).map_err(|e| format!("serialize baseline: {e}"))?;
    fs::write(&temporary, format!("{contents}\n"))
        .map_err(|e| format!("write {}: {e}", temporary.display()))?;
    fs::rename(&temporary, path).map_err(|e| format!("replace {}: {e}", path.display()))
}

fn stderr(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn binary_targets_are_sorted_by_name_then_package() {
        let metadata = Metadata {
            target_directory: "target".to_string(),
            packages: vec![
                Package {
                    name: "z-package".to_string(),
                    targets: vec![MetadataTarget {
                        name: "todo".to_string(),
                        kind: vec!["bin".to_string()],
                    }],
                },
                Package {
                    name: "a-package".to_string(),
                    targets: vec![MetadataTarget {
                        name: "alpha".to_string(),
                        kind: vec!["bin".to_string()],
                    }],
                },
            ],
        };
        let names = discover_binary_targets(&metadata)
            .into_iter()
            .map(|t| t.name)
            .collect::<Vec<_>>();
        assert_eq!(names, ["alpha", "todo"]);
    }

    #[test]
    fn percentile_uses_nearest_lower_rank() {
        assert_eq!(percentile(&[10, 20, 30, 40], 50), 20);
        assert_eq!(percentile(&[10, 20, 30, 40], 95), 30);
    }

    #[test]
    fn serialized_schema_has_stable_top_level_fields() {
        let json = serde_json::to_string(&Baseline {
            schema_version: 1,
            measured_at_unix_seconds_utc: 1,
            git_commit: "abc".to_string(),
            git_worktree_dirty: false,
            rustc: "rustc".to_string(),
            cargo: "cargo".to_string(),
            host: "host".to_string(),
            target: "target".to_string(),
            profile: "release".to_string(),
            build_arguments: vec!["build".to_string()],
            runs: 1,
            targets: vec![],
        })
        .unwrap();
        let value: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert!(json.find("schema_version").unwrap() < json.find("targets").unwrap());
    }
}
