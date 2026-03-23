//! Redirect handling, pipeline execution, and builtin dispatch.

mod builtin_impl;
mod workspace;

use std::io::Cursor;
use std::io::{Read, Write};

use super::super::parser::{Pipeline, SimpleCommand};
use super::super::vfs::Vfs;
use super::types::{BuiltinError, ExecContext, RunResult};

use builtin_impl::run_builtin_core;
use workspace::{workspace_read_file, workspace_write_file};

/// Maximum bytes buffered for a **non-terminal** pipeline stage’s stdout (host memory; design §8.2).
pub const PIPELINE_INTER_STAGE_MAX_BYTES: usize = 16 * 1024 * 1024;

#[inline]
const fn check_pipeline_inter_stage_size(len: usize) -> Result<(), BuiltinError> {
    if len > PIPELINE_INTER_STAGE_MAX_BYTES {
        return Err(BuiltinError::PipelineInterStageBufferExceeded {
            limit: PIPELINE_INTER_STAGE_MAX_BYTES,
            actual: len,
        });
    }
    Ok(())
}

/// Run a single command with given streams. Redirects override the provided stdin/stdout/stderr.
fn run_builtin_with_streams(
    vfs: &mut Vfs,
    vm_session: &mut super::super::vm::SessionHolder,
    default_stdin: &mut dyn Read,
    default_stdout: &mut dyn Write,
    default_stderr: &mut dyn Write,
    cmd: &SimpleCommand,
) -> Result<(), BuiltinError> {
    let redirects = &cmd.redirects;
    let argv = &cmd.argv;

    let mut stdin_override: Option<Cursor<Vec<u8>>> = None;
    let mut stdout_override: Option<Vec<u8>> = None;
    let mut stderr_override: Option<Vec<u8>> = None;
    let mut stdout_redirect_path: Option<String> = None;
    let mut stderr_redirect_path: Option<String> = None;

    for r in redirects {
        match r.fd {
            0 => {
                let content = workspace_read_file(vfs, vm_session, &r.path)?;
                stdin_override = Some(Cursor::new(content));
            }
            1 => {
                stdout_override = Some(Vec::new());
                stdout_redirect_path = Some(r.path.clone());
            }
            2 => {
                stderr_override = Some(Vec::new());
                stderr_redirect_path = Some(r.path.clone());
            }
            _ => {}
        }
    }

    let stdin: &mut dyn Read = stdin_override
        .as_mut()
        .map_or(default_stdin, |c| c as &mut dyn Read);
    let stdout: &mut dyn Write = stdout_override
        .as_mut()
        .map_or(default_stdout, |v| v as &mut dyn Write);
    let stderr: &mut dyn Write = stderr_override
        .as_mut()
        .map_or(default_stderr, |v| v as &mut dyn Write);

    let result = run_builtin_core(vfs, vm_session, stdin, stdout, stderr, argv);

    if let Some(path) = stdout_redirect_path {
        if let Some(buf) = &stdout_override {
            workspace_write_file(vfs, vm_session, &path, buf)?;
        }
    }
    if let Some(path) = stderr_redirect_path {
        if let Some(buf) = &stderr_override {
            workspace_write_file(vfs, vm_session, &path, buf)?;
        }
    }

    result
}

/// Apply redirects: build optional stdin (Cursor over file content), stdout/stderr buffers.
/// Then run the builtin with the effective streams; after that write stdout/stderr buffers to vfs if redirected.
///
/// # Errors
/// Returns `BuiltinError` on redirect or builtin execution failure.
pub fn run_builtin(ctx: &mut ExecContext<'_>, cmd: &SimpleCommand) -> Result<(), BuiltinError> {
    run_builtin_with_streams(
        ctx.vfs,
        ctx.vm_session,
        ctx.stdin,
        ctx.stdout,
        ctx.stderr,
        cmd,
    )
}

/// Execute a pipeline: run each command with stdin from previous stage (or `ctx.stdin` for first),
/// stdout to a buffer; last command's stdout is written to `ctx.stdout`. Redirects override pipe.
///
/// Non-final stages buffer **all** stdout in host memory; size is capped at
/// [`PIPELINE_INTER_STAGE_MAX_BYTES`] (design §8.2).
///
/// # Errors
/// Returns `BuiltinError` if any command or redirect fails.
///
/// # Panics
/// Panics if the pipeline state is inconsistent (non-first stage without pipe input); this is a programming error.
pub fn execute_pipeline(
    ctx: &mut ExecContext<'_>,
    pipeline: &Pipeline,
) -> Result<RunResult, BuiltinError> {
    let commands = &pipeline.commands;
    if commands.is_empty() {
        return Ok(RunResult::Continue);
    }

    let first_argv0 = commands
        .first()
        .and_then(|c| c.argv.first())
        .map(String::as_str);
    if first_argv0 == Some("exit") || first_argv0 == Some("quit") {
        return Ok(RunResult::Exit);
    }

    let mut prev_output: Option<Vec<u8>> = None;

    for (i, cmd) in commands.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == commands.len() - 1;

        let mut pipe_stdin: Option<Cursor<Vec<u8>>> = prev_output.take().map(Cursor::new);
        let mut next_buffer = Vec::new();

        let stdin: &mut dyn Read = if is_first {
            ctx.stdin
        } else {
            pipe_stdin
                .as_mut()
                .expect("non-first pipeline stage has pipe input") as &mut dyn Read
        };
        let stdout: &mut dyn Write = if is_last {
            ctx.stdout
        } else {
            &mut next_buffer
        };

        run_builtin_with_streams(ctx.vfs, ctx.vm_session, stdin, stdout, ctx.stderr, cmd)?;

        if !is_last {
            check_pipeline_inter_stage_size(next_buffer.len())?;
            prev_output = Some(next_buffer);
        }
    }

    Ok(RunResult::Continue)
}

#[cfg(test)]
mod pipeline_limit_tests {
    use super::*;

    #[test]
    fn pipeline_inter_stage_limit_boundary() {
        assert!(check_pipeline_inter_stage_size(PIPELINE_INTER_STAGE_MAX_BYTES).is_ok());
        let e = check_pipeline_inter_stage_size(PIPELINE_INTER_STAGE_MAX_BYTES + 1).unwrap_err();
        match e {
            BuiltinError::PipelineInterStageBufferExceeded { limit, actual } => {
                assert_eq!(limit, PIPELINE_INTER_STAGE_MAX_BYTES);
                assert_eq!(actual, PIPELINE_INTER_STAGE_MAX_BYTES + 1);
            }
            _ => panic!("expected PipelineInterStageBufferExceeded"),
        }
    }
}
