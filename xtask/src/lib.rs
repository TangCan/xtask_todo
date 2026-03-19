//! xtask library - logic for custom cargo tasks (testable).

mod clean;
mod clippy;
mod coverage;
mod fmt;
mod gh;
mod git;
mod publish;
mod run;
mod todo;

#[cfg(test)]
mod tests;

use argh::FromArgs;

use crate::todo::TodoArgs;
use clean::CleanArgs;
use clippy::ClippyArgs;
use coverage::CoverageArgs;
use fmt::FmtArgs;
use gh::GhArgs;
use git::GitArgs;
use publish::PublishArgs;
use run::RunArgs;

/// Run failure with exit code (0 = success; 1 general, 2 parameter, 3 data for todo).
#[derive(Debug)]
pub struct RunFailure {
    pub code: i32,
    pub message: String,
}

/// Entry point for xtask. Parses args and runs the selected command.
///
/// # Errors
/// Returns `RunFailure` with appropriate exit code on error.
pub fn run() -> Result<(), RunFailure> {
    let cmd: XtaskCmd = argh::from_env();
    run_with(cmd)
}

/// Run with a pre-parsed command (for tests).
///
/// # Errors
/// Returns `RunFailure` with code 1 for most subcommands; todo uses 2 (parameter) or 3 (data).
pub fn run_with(cmd: XtaskCmd) -> Result<(), RunFailure> {
    match cmd.sub {
        XtaskSub::Run(args) => {
            run::cmd_run(args);
            Ok(())
        }
        XtaskSub::Clean(args) => clean::cmd_clean(args).map_err(|e| to_run_failure(&*e)),
        XtaskSub::Clippy(args) => clippy::cmd_clippy(args).map_err(|e| to_run_failure(&*e)),
        XtaskSub::Coverage(args) => coverage::cmd_coverage(args).map_err(|e| to_run_failure(&*e)),
        XtaskSub::Fmt(args) => fmt::cmd_fmt(args).map_err(|e| to_run_failure(&*e)),
        XtaskSub::Gh(args) => gh::cmd_gh(&args).map_err(|e| to_run_failure(&*e)),
        XtaskSub::Git(args) => git::cmd_git(&args).map_err(|e| to_run_failure(&*e)),
        XtaskSub::Publish(args) => publish::cmd_publish(&args).map_err(|e| to_run_failure(&*e)),
        XtaskSub::Todo(args) => todo::cmd_todo(args).map_err(|e| RunFailure {
            code: e.exit_code(),
            message: e.to_string(),
        }),
    }
}

fn to_run_failure(e: &(dyn std::error::Error + 'static)) -> RunFailure {
    RunFailure {
        code: 1,
        message: e.to_string(),
    }
}

#[derive(FromArgs, Clone)]
/// Cargo xtask - custom build/tooling tasks
pub struct XtaskCmd {
    #[argh(subcommand)]
    pub sub: XtaskSub,
}

#[derive(FromArgs, Clone)]
#[argh(subcommand)]
pub enum XtaskSub {
    Run(RunArgs),
    Clean(CleanArgs),
    Clippy(ClippyArgs),
    Coverage(CoverageArgs),
    Fmt(FmtArgs),
    Gh(GhArgs),
    Git(GitArgs),
    Publish(PublishArgs),
    Todo(TodoArgs),
}
