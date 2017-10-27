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
    static ref HANDLER: Mutex<Option<Box<Fn(Signal) -> HandleOutcome + Send>>> =
        Mutex::new(None);
}

pub enum Signal {
    /// Represent CTRL_C_EVENT (0)
    CtrlCEvent,

    /// Represent CTRL_BREAK_EVENT (1)
    CtrlBreakEvent,

    /// Represent CTRL_CLOSE_EVENT (2)
    CtrlCloseEvent,

    /// Represent CTRL_LOGOFF_EVENT (5)
    CtrlLogoffEvent,

    /// Represent CTRL_SHUTDOWN_EVENT (6)
    CtrlShutdownEvent,
}

pub enum HandleOutcome {
    Handled,
    Passthrough,
}

pub type HandleResult = Result<(), ()>;

pub fn set_handler<F>(f: F) -> HandleResult
where
    F: 'static + Send + Fn(Signal) -> HandleOutcome,
{
    let mut handler = HANDLER.lock().unwrap();
    *handler = Some(Box::new(f));

    set_console_ctrl_handler_wrap(Some(sig_handler), TRUE)
}

pub fn reset() -> HandleResult {
    set_console_ctrl_handler_wrap(None, FALSE)
}

fn set_console_ctrl_handler_wrap(handler_routine: PHANDLER_ROUTINE, add: BOOL) -> HandleResult {
    let ret = unsafe { SetConsoleCtrlHandler(handler_routine, add) };
    if ret != 0 { Ok(()) } else { Err(()) }
}

unsafe extern "system" fn sig_handler(event: DWORD) -> BOOL {
    let sig = match event {
        CTRL_C_EVENT => Signal::CtrlCEvent,
        CTRL_BREAK_EVENT => Signal::CtrlBreakEvent,
        CTRL_CLOSE_EVENT => Signal::CtrlCloseEvent,
        CTRL_LOGOFF_EVENT => Signal::CtrlLogoffEvent,
        CTRL_SHUTDOWN_EVENT => Signal::CtrlShutdownEvent,
        _ => Signal::CtrlCEvent,
    };

    if let Some(ref handler) = *HANDLER.lock().unwrap() {
        match handler(sig) {
            HandleOutcome::Handled => TRUE,
            HandleOutcome::Passthrough => FALSE,
        }
    } else {
        FALSE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sig_handler_ctrl_c() {
        let res = set_handler(|sig| match sig {
            Signal::CtrlCEvent => HandleOutcome::Handled,
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
