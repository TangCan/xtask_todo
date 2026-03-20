//! Script execution: parse script to AST (assign, command, if/for/while, source), then interpret.

mod ast;
mod exec;
mod parse;

#[cfg(test)]
mod tests;

pub use ast::{ParseError, ScriptStmt};
pub use exec::{expand_vars, logical_lines, read_script_source_text, run_script, CmdOutcome};
pub use parse::parse_script;
