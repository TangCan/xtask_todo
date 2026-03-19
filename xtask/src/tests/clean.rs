//! Tests for clean subcommand.

use crate::clean::{cmd_clean, CleanArgs};
use crate::tests::{cwd_test_lock, dir_outside_cwd, RestoreCwd};
use crate::{run_with, XtaskCmd, XtaskSub};

#[test]
fn cmd_clean_removes_xtask_prefix_dirs() {
    let _lock = cwd_test_lock();
    let root = dir_outside_cwd("xtask_clean_test");
    std::fs::create_dir_all(&root).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&root, &cwd);

    let d1 = root.join("xtask_clippy_foo");
    let d2 = root.join("xtask_git_bar");
    std::fs::create_dir_all(&d1).unwrap();
    std::fs::create_dir_all(&d2).unwrap();
    assert!(d1.exists());
    assert!(d2.exists());

    let r = cmd_clean(CleanArgs {});
    assert!(r.is_ok(), "{r:?}");
    assert!(!d1.exists());
    assert!(!d2.exists());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn cmd_clean_no_matching_dirs_prints_message() {
    let _lock = cwd_test_lock();
    let root = dir_outside_cwd("xtask_clean_empty");
    std::fs::create_dir_all(&root).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&root, &cwd);

    let r = cmd_clean(CleanArgs {});
    assert!(r.is_ok(), "{r:?}");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn run_with_clean_success() {
    let _lock = cwd_test_lock();
    let root = dir_outside_cwd("xtask_clean_runwith");
    std::fs::create_dir_all(&root).unwrap();
    let cwd = std::env::current_dir().unwrap();
    let _restore = RestoreCwd::new(&root, &cwd);

    let d = root.join("xtask_nongit_xyz");
    std::fs::create_dir_all(&d).unwrap();
    let cmd = XtaskCmd {
        sub: XtaskSub::Clean(CleanArgs {}),
    };
    let r = run_with(cmd);
    assert!(r.is_ok(), "{r:?}");
    assert!(!d.exists());

    let _ = std::fs::remove_dir_all(&root);
}
