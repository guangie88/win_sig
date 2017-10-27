extern crate kernel32;
extern crate winapi;
#[macro_use]
extern crate lazy_static;

use std::sync::{Arc, Mutex};
use kernel32::SetConsoleCtrlHandler;
use winapi::minwindef::{BOOL, DWORD, FALSE, TRUE};
use winapi::wincon::{CTRL_C_EVENT, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_LOGOFF_EVENT,
                     CTRL_SHUTDOWN_EVENT};

lazy_static! {
    static ref HANDLER: Arc<Mutex<Option<Box<Fn(Signal) -> HandleOutcome + Send>>>> =
        Arc::new(Mutex::new(None));
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

pub fn set_handler<F>(f: F)
    where F: 'static + Send + Fn(Signal) -> HandleOutcome
{
    let handler = HANDLER.clone();
    let mut handler = handler.lock().unwrap();
    *handler = Some(Box::new(f));

    unsafe {
        SetConsoleCtrlHandler(Some(sig_handler), TRUE);
    }
}

pub fn reset() {
    unsafe { SetConsoleCtrlHandler(None, FALSE); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sig_handler_ctrl_c() {
        set_handler(|sig| match sig {
                        Signal::CtrlCEvent => HandleOutcome::Handled,
                        _ => HandleOutcome::Passthrough,
                    })
    }

    #[test]
    fn test_sig_handler_handled() {
        set_handler(|_| HandleOutcome::Handled);
    }

    #[test]
    fn test_sig_handler_passthrough() {
        set_handler(|_| HandleOutcome::Passthrough);
    }
}
