//! `lima-todo` xtask: build standalone `todo` and merge Lima `mounts:` + `PATH` into `lima.yaml`.

mod args;
mod cmd;
mod helpers;
mod yaml;

#[cfg(test)]
mod tests;

pub use args::LimaTodoArgs;
pub use cmd::cmd_lima_todo;
