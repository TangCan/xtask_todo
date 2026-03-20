//! Rustyline `Helper`: `Completer`, `Hinter`, `Highlighter`, `Validator` for devshell.

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hint, Hinter};
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Context;
use rustyline::Helper;

use super::super::vfs::Vfs;
use super::super::vm::SessionHolder;
use super::candidates::{complete_commands, complete_path, list_dir_names_for_completion};
use super::context::{completion_context, CompletionKind};

/// Helper for rustyline: command and path tab-completion using shared [`Vfs`] and [`SessionHolder`].
pub struct DevShellHelper {
    pub vfs: Rc<RefCell<Vfs>>,
    pub vm_session: Rc<RefCell<SessionHolder>>,
}

impl DevShellHelper {
    #[must_use]
    pub const fn new(vfs: Rc<RefCell<Vfs>>, vm_session: Rc<RefCell<SessionHolder>>) -> Self {
        Self { vfs, vm_session }
    }
}

/// Dummy hint type for Hinter; we never return a hint (`hint()` returns None).
#[derive(Debug)]
pub struct NoHint;
impl Hint for NoHint {
    fn display(&self) -> &'static str {
        ""
    }
    fn completion(&self) -> Option<&str> {
        None
    }
}

impl Completer for DevShellHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<String>), rustyline::error::ReadlineError> {
        let Some(ctx) = completion_context(line, pos) else {
            return Ok((pos, vec![]));
        };
        let candidates = match ctx.kind {
            CompletionKind::Command => complete_commands(&ctx.prefix),
            CompletionKind::Other => vec![],
            CompletionKind::Path => {
                let parent = if ctx.prefix.contains('/') {
                    let idx = ctx.prefix.rfind('/').unwrap();
                    if idx == 0 {
                        "/".to_string()
                    } else {
                        ctx.prefix[..idx].to_string()
                    }
                } else {
                    ".".to_string()
                };
                let abs_parent = self.vfs.borrow().resolve_to_absolute(&parent);
                let names = list_dir_names_for_completion(&self.vfs, &self.vm_session, &abs_parent);
                complete_path(&ctx.prefix, &names)
            }
        };
        Ok((ctx.start, candidates))
    }
}

impl Hinter for DevShellHelper {
    type Hint = NoHint;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<NoHint> {
        None
    }
}

impl Highlighter for DevShellHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }
}

impl Validator for DevShellHelper {
    fn validate(
        &self,
        _ctx: &mut ValidationContext<'_>,
    ) -> Result<ValidationResult, rustyline::error::ReadlineError> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for DevShellHelper {}
