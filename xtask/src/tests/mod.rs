//! Unit tests for xtask commands.

mod clippy;
mod git;
mod run;
mod todo;

use std::path::PathBuf;
use std::sync::Mutex;

/// Serializes tests that change `current_dir` so they don't race (`current_dir` is process-global).
/// Uses `into_inner()` on poison so one panicking test doesn't cause all others to fail with `PoisonError`.
pub static CWD_TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Acquires the CWD mutex; if poisoned (a prior test panicked while holding it), continues with the inner lock.
pub fn cwd_test_lock() -> std::sync::MutexGuard<'static, ()> {
    CWD_TEST_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

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
