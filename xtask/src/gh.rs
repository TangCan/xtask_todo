//! `gh` subcommand - show GitHub Actions run log (e.g. `gh log` = latest run log).

use argh::FromArgs;
use serde_json::Value as JsonValue;
use std::process::Command;

/// Default workflow file when filtering by job (matches `.github/workflows/release.yml`).
const DEFAULT_WORKFLOW_FOR_JOB: &str = "release.yml";

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "gh")]
/// GitHub CLI helpers (e.g. show latest Actions run log)
pub struct GhArgs {
    #[argh(subcommand)]
    pub sub: GhSub,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand)]
pub enum GhSub {
    Log(GhLogArgs),
}

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "log")]
/// Show log of the most recent GitHub Actions run, or a single job (e.g. devshell-vm-oci) from the latest run of workflow release.yml
pub struct GhLogArgs {
    /// show only this job's log (e.g. devshell-vm-oci); uses latest run of --workflow (default: release.yml)
    #[argh(option, short = 'j', long = "job")]
    pub job: Option<String>,
    /// workflow file for run list when --job is set (default: release.yml)
    #[argh(option, short = 'w', long = "workflow")]
    pub workflow: Option<String>,
}

/// Run gh subcommand.
///
/// # Errors
/// Returns an error if `gh` is not found, list returns no run, or view fails; prints message to stderr.
pub fn cmd_gh(args: &GhArgs) -> Result<(), Box<dyn std::error::Error>> {
    match &args.sub {
        GhSub::Log(a) => cmd_gh_log(a),
    }
}

fn cmd_gh_log(args: &GhLogArgs) -> Result<(), Box<dyn std::error::Error>> {
    args.job
        .as_ref()
        .map_or_else(cmd_gh_log_latest_any_workflow, |job_name| {
            cmd_gh_log_job(
                job_name.trim(),
                args.workflow
                    .as_deref()
                    .unwrap_or(DEFAULT_WORKFLOW_FOR_JOB)
                    .trim(),
            )
        })
}

fn json_number_to_u64(id: &JsonValue) -> Option<u64> {
    id.as_u64()
        .or_else(|| id.as_i64().and_then(|i| u64::try_from(i).ok()))
}

fn cmd_gh_log_latest_any_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let out = Command::new("gh")
        .args([
            "run",
            "list",
            "--limit",
            "1",
            "--json",
            "databaseId",
            "-q",
            ".[0].databaseId",
        ])
        .output();

    let out = match out {
        Ok(o) => o,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("gh: command not found");
            } else {
                eprintln!("gh: {e}");
            }
            return Err(e.into());
        }
    };

    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
        if msg.is_empty() {
            eprintln!("gh run list failed");
        } else {
            eprintln!("{msg}");
        }
        return Err("gh run list failed".into());
    }

    let id = String::from_utf8(out.stdout)
        .map_err(|_| "gh run list produced invalid UTF-8")?
        .trim()
        .to_string();

    if id.is_empty() {
        eprintln!("no runs found");
        return Err("no runs found".into());
    }

    let status = Command::new("gh")
        .args(["run", "view", &id, "--log"])
        .status()?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        eprintln!("gh run view exited with code {code}");
        return Err(format!("gh run view failed (exit code {code})").into());
    }

    Ok(())
}

fn gh_repo_name_with_owner() -> Result<String, Box<dyn std::error::Error>> {
    let out = Command::new("gh")
        .args([
            "repo",
            "view",
            "--json",
            "nameWithOwner",
            "-q",
            ".nameWithOwner",
        ])
        .output();
    let out = match out {
        Ok(o) => o,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("gh: command not found");
            } else {
                eprintln!("gh: {e}");
            }
            return Err(e.into());
        }
    };
    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
        if msg.is_empty() {
            eprintln!("gh repo view failed");
        } else {
            eprintln!("{msg}");
        }
        return Err("gh repo view failed".into());
    }
    let s = String::from_utf8(out.stdout)?.trim().to_string();
    if s.is_empty() {
        return Err("gh repo view: empty nameWithOwner".into());
    }
    Ok(s)
}

fn cmd_gh_log_job(job_name: &str, workflow_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    if job_name.is_empty() {
        eprintln!("--job must not be empty");
        return Err("--job must not be empty".into());
    }
    if workflow_file.is_empty() {
        eprintln!("--workflow must not be empty");
        return Err("--workflow must not be empty".into());
    }

    let out = Command::new("gh")
        .args([
            "run",
            "list",
            "--workflow",
            workflow_file,
            "--limit",
            "1",
            "--json",
            "databaseId",
            "-q",
            ".[0].databaseId",
        ])
        .output();

    let out = match out {
        Ok(o) => o,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("gh: command not found");
            } else {
                eprintln!("gh: {e}");
            }
            return Err(e.into());
        }
    };

    if !out.status.success() {
        let msg = String::from_utf8_lossy(&out.stderr).trim().to_string();
        if msg.is_empty() {
            eprintln!("gh run list failed");
        } else {
            eprintln!("{msg}");
        }
        return Err("gh run list failed".into());
    }

    let run_id = String::from_utf8(out.stdout)?.trim().to_string();
    if run_id.is_empty() {
        eprintln!("no workflow runs found for workflow {workflow_file:?}");
        return Err("no workflow runs found".into());
    }

    let repo = gh_repo_name_with_owner()?;
    let path = format!("repos/{repo}/actions/runs/{run_id}/jobs");
    let api_out = Command::new("gh").args(["api", &path]).output()?;
    if !api_out.status.success() {
        let msg = String::from_utf8_lossy(&api_out.stderr).trim().to_string();
        if msg.is_empty() {
            eprintln!("gh api {path} failed");
        } else {
            eprintln!("{msg}");
        }
        return Err("gh api jobs failed".into());
    }

    let body: JsonValue = serde_json::from_slice(&api_out.stdout)?;
    let jobs = body
        .get("jobs")
        .and_then(|j| j.as_array())
        .ok_or("jobs API: missing jobs array")?;

    let job_id = jobs
        .iter()
        .find(|j| j.get("name").and_then(|n| n.as_str()) == Some(job_name))
        .and_then(|j| j.get("id"))
        .and_then(json_number_to_u64)
        .ok_or_else(|| {
            format!("no job named {job_name:?} in run {run_id} (workflow {workflow_file:?})")
        })?;

    let status = Command::new("gh")
        .args([
            "run",
            "view",
            &run_id,
            "--job",
            &job_id.to_string(),
            "--log",
        ])
        .status()?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        eprintln!("gh run view exited with code {code}");
        return Err(format!("gh run view failed (exit code {code})").into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{json_number_to_u64, JsonValue};

    #[test]
    fn json_number_to_u64_prefers_u64() {
        let v = serde_json::json!(68_155_411_347_u64);
        assert_eq!(json_number_to_u64(&v), Some(68_155_411_347_u64));
    }

    #[test]
    fn json_number_to_u64_accepts_positive_i64() {
        let v = JsonValue::from(42_i64);
        assert_eq!(json_number_to_u64(&v), Some(42_u64));
    }

    #[test]
    fn json_number_to_u64_rejects_negative_i64() {
        let v = JsonValue::from(-1_i64);
        assert_eq!(json_number_to_u64(&v), None);
    }
}
