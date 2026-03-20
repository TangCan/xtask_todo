//! Restore Unix execute bits on ELF artifacts under `target/` after VFS sync.

use std::path::Path;

use super::error::SandboxError;

const MAX_TARGET_TREE_DEPTH: usize = 64;

#[cfg(unix)]
fn host_file_starts_with_elf_magic(path: &Path) -> Result<bool, SandboxError> {
    use std::fs::File;
    use std::io::ErrorKind;
    use std::io::Read;

    let meta = std::fs::metadata(path).map_err(SandboxError::ExportFailed)?;
    if !meta.is_file() || meta.len() < 4 {
        return Ok(false);
    }
    let mut f = File::open(path).map_err(SandboxError::ExportFailed)?;
    let mut magic = [0u8; 4];
    match f.read_exact(&mut magic) {
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(false),
        Err(e) => return Err(SandboxError::ExportFailed(e)),
    }
    Ok(magic == [0x7F, b'E', b'L', b'F'])
}

#[cfg(unix)]
fn restore_elf_execute_bits_under_target(dir: &Path, depth: usize) -> Result<(), SandboxError> {
    use std::os::unix::fs::PermissionsExt;

    if depth > MAX_TARGET_TREE_DEPTH {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir).map_err(SandboxError::ExportFailed)? {
        let entry = entry.map_err(SandboxError::ExportFailed)?;
        let path = entry.path();
        let ty = entry.file_type().map_err(SandboxError::ExportFailed)?;
        if ty.is_dir() {
            restore_elf_execute_bits_under_target(&path, depth + 1)?;
        } else if ty.is_file() && host_file_starts_with_elf_magic(&path)? {
            let mut perms = path
                .metadata()
                .map_err(SandboxError::ExportFailed)?
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms).map_err(SandboxError::ExportFailed)?;
        }
    }
    Ok(())
}

/// Restore `0755` on ELF files under `project_root/target` so `cargo run` can exec them after a VFS
/// export (which uses [`std::fs::write`] and drops the Unix execute bit).
#[cfg(unix)]
pub fn restore_execute_bits_for_build_artifacts(project_root: &Path) -> Result<(), SandboxError> {
    let target = project_root.join("target");
    if target.is_dir() {
        restore_elf_execute_bits_under_target(&target, 0)?;
    }
    Ok(())
}

#[cfg(not(unix))]
pub fn restore_execute_bits_for_build_artifacts(_project_root: &Path) -> Result<(), SandboxError> {
    Ok(())
}
