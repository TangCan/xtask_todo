use std::process::Command;

#[must_use]
pub fn xtask_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
}

#[must_use]
pub fn todo_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_todo"))
}
