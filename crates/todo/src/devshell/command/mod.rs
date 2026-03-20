//! Command execution: `ExecContext`, redirects, and builtin dispatch (pwd, cd, ls, mkdir, cat, touch, echo, export-readonly, save, todo, exit/quit).

mod dispatch;
mod todo_builtin;
mod types;

pub use dispatch::{execute_pipeline, run_builtin, PIPELINE_INTER_STAGE_MAX_BYTES};
pub use types::{BuiltinError, ExecContext, RunResult};
