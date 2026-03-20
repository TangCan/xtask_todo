//! Tab completion: context parsing (command vs path), command and path candidates.
//! Rustyline Helper (Completer, Hinter, Highlighter, Validator) for command and path completion.

mod candidates;
mod context;
mod helper;

#[cfg(test)]
mod tests;

pub use candidates::{complete_commands, complete_path, list_dir_names_for_completion};
pub use context::{completion_context, CompletionContext, CompletionKind};
pub use helper::{DevShellHelper, NoHint};
