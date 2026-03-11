//! xtask library - logic for custom cargo tasks (testable).

mod clippy;
mod coverage;
mod fmt;
mod git;
mod run;
mod todo;

#[cfg(test)]
mod tests;

use argh::FromArgs;

use clippy::ClippyArgs;
use coverage::CoverageArgs;
use fmt::FmtArgs;
use git::GitArgs;
use run::RunArgs;
use todo::TodoArgs;

/// Entry point for xtask. Parses args and runs the selected command.
///
/// # Errors
/// Propagates errors from the selected subcommand (e.g. clippy, git, todo).
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cmd: XtaskCmd = argh::from_env();
    run_with(cmd)
}

/// Run with a pre-parsed command (for tests).
///
/// # Errors
/// Propagates errors from the selected subcommand.
pub fn run_with(cmd: XtaskCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.sub {
        XtaskSub::Run(args) => {
            run::cmd_run(args);
            Ok(())
        }
        XtaskSub::Clippy(args) => clippy::cmd_clippy(args),
        XtaskSub::Coverage(args) => coverage::cmd_coverage(args),
        XtaskSub::Fmt(args) => fmt::cmd_fmt(args),
        XtaskSub::Git(args) => git::cmd_git(&args),
        XtaskSub::Todo(args) => todo::cmd_todo(args),
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
    Clippy(ClippyArgs),
    Coverage(CoverageArgs),
    Fmt(FmtArgs),
    Git(GitArgs),
    Todo(TodoArgs),
}
