//! xtask - custom cargo tasks
//!
//! Run with: cargo xtask <command>

use argh::FromArgs;

fn main() {
    let cmd: XtaskCmd = argh::from_env();
    if let Err(e) = run(cmd) {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run(cmd: XtaskCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd.sub {
        XtaskSub::Run(args) => cmd_run(args),
    }
}

#[derive(FromArgs)]
/// Cargo xtask - custom build/tooling tasks
struct XtaskCmd {
    #[argh(subcommand)]
    sub: XtaskSub,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum XtaskSub {
    Run(RunArgs),
}

#[derive(FromArgs)]
#[argh(subcommand, name = "run")]
/// Run the main project (example task)
struct RunArgs {}

fn cmd_run(_args: RunArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("xtask run: placeholder - add your task logic here");
    Ok(())
}
