//! `run` subcommand - placeholder example task.

use argh::FromArgs;

#[derive(FromArgs, Clone)]
#[argh(subcommand, name = "run")]
/// Run the main project (example task)
pub struct RunArgs {}

pub fn cmd_run(_args: RunArgs) {
    println!("xtask run: placeholder - add your task logic here");
}
