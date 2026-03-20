//! Optional Linux mount namespace for sandbox child (no container engine).

use std::process::Command;

pub(super) fn linux_mount_namespace_enabled() -> bool {
    std::env::var("DEVSHELL_RUST_MOUNT_NAMESPACE").is_ok_and(|s| {
        let s = s.trim();
        s == "1" || s.eq_ignore_ascii_case("true") || s.eq_ignore_ascii_case("yes")
    })
}

/// Child enters a new mount namespace and marks the tree private (no Podman/Docker).
pub(super) fn apply_linux_private_mount_namespace(cmd: &mut Command) {
    use std::os::unix::process::CommandExt;

    unsafe {
        cmd.pre_exec(|| {
            if libc::unshare(libc::CLONE_NEWNS) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let target = b"/\0";
            let r = libc::mount(
                std::ptr::null(),
                target.as_ptr().cast::<libc::c_char>(),
                std::ptr::null(),
                libc::MS_REC | libc::MS_PRIVATE,
                std::ptr::null(),
            );
            if r != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}
