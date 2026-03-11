//! Unit tests for xtask commands.

mod clippy;
mod git;
mod run;
mod todo;

use std::path::PathBuf;
use std::sync::Mutex;

/// Serializes tests that change `current_dir` so they don't race (`current_dir` is process-global).
pub static CWD_TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Restores the current directory when dropped (used by tests that change cwd).
pub struct RestoreCwd(PathBuf);
impl RestoreCwd {
    pub fn new(dir: &std::path::Path, cwd: &std::path::Path) -> Self {
        std::env::set_current_dir(dir).unwrap();
        Self(cwd.to_path_buf())
    }
}
impl Drop for RestoreCwd {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}
