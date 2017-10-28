extern crate kernel32;
extern crate winapi;
#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use kernel32::SetConsoleCtrlHandler;
use winapi::minwindef::{BOOL, DWORD, FALSE, TRUE};
use winapi::wincon::{CTRL_C_EVENT, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT,
                     CTRL_SHUTDOWN_EVENT, PHANDLER_ROUTINE};

lazy_static! {
    static ref HANDLER: Mutex<Option<Box<Fn(CtrlEvent) -> HandleOutcome + Send>>> =
        Mutex::new(None);
}

#[derive(Clone, Copy)]
pub enum CtrlEvent {
    /// Represent CTRL_C_EVENT (0)
    C,

    /// Represent CTRL_BREAK_EVENT (1)
    Break,

    /// Represent CTRL_CLOSE_EVENT (2)
    Close,

    /// Represent CTRL_LOGOFF_EVENT (5)
    Logoff,

    /// Represent CTRL_SHUTDOWN_EVENT (6)
    Shutdown,
}

#[derive(Clone)]
pub enum HandleError {
    Lock,
    Os,
}

pub enum HandleOutcome {
    Handled,
    Passthrough,
}

pub type HandleResult = Result<(), HandleError>;

pub fn set_handler<F>(f: F) -> HandleResult
where
    F: 'static + Send + Fn(CtrlEvent) -> HandleOutcome,
{
    match HANDLER.try_lock() {
        Ok(mut handler) => {
            *handler = Some(Box::new(f));
            set_console_ctrl_handler_wrap(Some(sig_handler), TRUE)
        },
        Err(_) => Err(HandleError::Lock),
    }
}

pub fn reset() -> HandleResult {
    set_console_ctrl_handler_wrap(None, FALSE)
}

fn set_console_ctrl_handler_wrap(handler_routine: PHANDLER_ROUTINE, add: BOOL) -> HandleResult {
    let ret = unsafe { SetConsoleCtrlHandler(handler_routine, add) };
    if ret != 0 { Ok(()) } else { Err(HandleError::Os) }
}

unsafe extern "system" fn sig_handler(event: DWORD) -> BOOL {
    let sig = match event {
        CTRL_C_EVENT => Some(CtrlEvent::C),
        CTRL_BREAK_EVENT => Some(CtrlEvent::Break),
        CTRL_CLOSE_EVENT => Some(CtrlEvent::Close),
        CTRL_LOGOFF_EVENT => Some(CtrlEvent::Logoff),
        CTRL_SHUTDOWN_EVENT => Some(CtrlEvent::Shutdown),
        _ => None,
    };

    // if the handler mutex cannot be locked or if the signal received is not part of list
    // simply do not handle and passthrough
    match HANDLER.try_lock() {
        Ok(handler_guard) => {
            if let (&Some(ref handler), Some(ref sig)) = (&*handler_guard, sig) {
                match handler(*sig) {
                    HandleOutcome::Handled => TRUE,
                    HandleOutcome::Passthrough => FALSE,
                }
            } else {
                FALSE
            }
        },
        Err(_) => FALSE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sig_handler_ctrl_c() {
        let res = set_handler(|sig| match sig {
            CtrlEvent::C => HandleOutcome::Handled,
            _ => HandleOutcome::Passthrough,
        });

        assert!(res.is_ok());
    }

    #[test]
    fn test_sig_handler_handled() {
        let res = set_handler(|_| HandleOutcome::Handled);
        assert!(res.is_ok());
    }

    #[test]
    fn test_sig_handler_passthrough() {
        let res = set_handler(|_| HandleOutcome::Passthrough);
        assert!(res.is_ok());
    }
}
