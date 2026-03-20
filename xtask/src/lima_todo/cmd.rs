//! `lima-todo` command entry and pre-flight checks.

#![allow(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    clippy::if_not_else
)]

use crate::RunFailure;

use super::args::{default_lima_yaml_path, lima_instance_name, LimaTodoArgs};
use super::helpers::{
    backup_and_write, backup_path, build_todo_release, cargo_metadata_target,
    host_release_str_for_target_dir, limactl_restart,
};
use super::yaml::{merge_todo_into_lima_yaml, render_fragment};

#[cfg(unix)]
/// Before build/write: if `lima.yaml` already has this mount + PATH, print and skip.
/// Returns `false` when the caller should exit successfully without building.
fn lima_todo_already_installed(
    args: &LimaTodoArgs,
    host_release_str: &str,
) -> Result<bool, RunFailure> {
    if args.print_only {
        return Ok(false);
    }
    let instance = lima_instance_name(args);
    let lima_yaml = args
        .lima_yaml
        .clone()
        .or_else(|| default_lima_yaml_path(&instance))
        .ok_or_else(|| RunFailure {
            code: 1,
            message: "could not resolve lima.yaml path (set HOME or --lima-yaml)".to_string(),
        })?;

    if !lima_yaml.is_file() {
        return Ok(false);
    }

    let original = std::fs::read_to_string(&lima_yaml).map_err(|e| RunFailure {
        code: 1,
        message: format!("read {}: {e}", lima_yaml.display()),
    })?;

    let (_merged, changed) =
        merge_todo_into_lima_yaml(&original, host_release_str, &args.guest_mount).map_err(|m| {
            RunFailure {
                code: 1,
                message: m,
            }
        })?;

    if !changed {
        println!(
            "lima-todo: already installed — {} already lists mount `{}` and env.PATH includes {:?}; nothing to do.",
            lima_yaml.display(),
            host_release_str,
            args.guest_mount
        );
        return Ok(true);
    }
    Ok(false)
}

/// Build standalone `todo`, merge into `lima.yaml` or print fragment.
pub fn cmd_lima_todo(args: LimaTodoArgs) -> Result<(), RunFailure> {
    let workspace = std::env::current_dir().map_err(|e| RunFailure {
        code: 1,
        message: format!("current_dir: {e}"),
    })?;

    let (_root, target_dir) = cargo_metadata_target(&workspace).map_err(|m| RunFailure {
        code: 1,
        message: m,
    })?;

    let host_release_str =
        host_release_str_for_target_dir(&target_dir).map_err(|m| RunFailure {
            code: 1,
            message: m,
        })?;

    #[cfg(unix)]
    {
        if lima_todo_already_installed(&args, &host_release_str)? {
            return Ok(());
        }
    }

    if !args.no_build {
        build_todo_release(&workspace).map_err(|m| RunFailure {
            code: 1,
            message: m,
        })?;
    }

    let release_dir = target_dir.join("release");
    let todo_bin = release_dir.join("todo");
    if !todo_bin.is_file() {
        return Err(RunFailure {
            code: 1,
            message: format!(
                "missing {} — run without --no-build or: cargo build -p xtask --release --bin todo",
                todo_bin.display()
            ),
        });
    }

    let fragment = render_fragment(&release_dir, &args.guest_mount).map_err(|m| RunFailure {
        code: 1,
        message: m,
    })?;

    if let Some(path) = &args.write {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| RunFailure {
                code: 1,
                message: format!("create_dir_all {}: {e}", parent.display()),
            })?;
        }
        std::fs::write(path, fragment.as_bytes()).map_err(|e| RunFailure {
            code: 1,
            message: format!("write {}: {e}", path.display()),
        })?;
        println!("Wrote Lima fragment to {}", path.display());
    }

    if args.print_only {
        if args.write.is_none() {
            print!("{fragment}");
        }
        return Ok(());
    }

    #[cfg(not(unix))]
    {
        return Err(RunFailure {
            code: 1,
            message:
                "lima.yaml merge is only supported on Unix; use --print-only and merge manually"
                    .to_string(),
        });
    }

    #[cfg(unix)]
    {
        let instance = lima_instance_name(&args);
        let lima_yaml = args
            .lima_yaml
            .clone()
            .or_else(|| default_lima_yaml_path(&instance))
            .ok_or_else(|| RunFailure {
                code: 1,
                message: "could not resolve lima.yaml path (set HOME or --lima-yaml)".to_string(),
            })?;

        if !lima_yaml.is_file() {
            return Err(RunFailure {
                code: 1,
                message: format!(
                    "lima.yaml not found: {}\n\
                     Create the Lima instance first (e.g. `limactl start -y {instance}`), or pass `--lima-yaml /path/to/lima.yaml`.",
                    lima_yaml.display()
                ),
            });
        }

        let original = std::fs::read_to_string(&lima_yaml).map_err(|e| RunFailure {
            code: 1,
            message: format!("read {}: {e}", lima_yaml.display()),
        })?;

        let (merged, changed) =
            merge_todo_into_lima_yaml(&original, &host_release_str, &args.guest_mount).map_err(
                |m| RunFailure {
                    code: 1,
                    message: m,
                },
            )?;

        if !changed {
            println!(
                "lima-todo: {} already had this `todo` mount and PATH; no file changes.",
                lima_yaml.display()
            );
        } else {
            backup_and_write(&lima_yaml, merged.as_bytes()).map_err(|m| RunFailure {
                code: 1,
                message: m,
            })?;
            println!(
                "lima-todo: updated {} (backup: {}), instance {:?}",
                lima_yaml.display(),
                backup_path(&lima_yaml).display(),
                instance
            );
        }

        if !args.no_restart {
            if changed {
                println!(
                    "lima-todo: restarting Lima instance {instance:?} (limactl stop / start -y)…"
                );
                limactl_restart(&instance).map_err(|m| RunFailure {
                    code: 1,
                    message: m,
                })?;
                println!(
                    "lima-todo: instance {instance:?} is running; in guest run `todo` or `command -v todo`."
                );
            } else {
                println!(
                    "lima-todo: skipped VM restart (lima.yaml unchanged). If `todo` is still missing in guest, run: limactl stop {instance} && limactl start -y {instance}"
                );
            }
        } else {
            println!(
                "lima-todo: skipped restart (--no-restart). Run: limactl stop {instance} && limactl start -y {instance}"
            );
        }

        Ok(())
    }
}
