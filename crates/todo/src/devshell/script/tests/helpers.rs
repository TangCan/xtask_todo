use std::cell::RefCell;
use std::rc::Rc;

use crate::devshell::vm::SessionHolder;

pub(super) fn vm_session_test() -> Rc<RefCell<SessionHolder>> {
    Rc::new(RefCell::new(SessionHolder::new_host()))
}
