// executable must be run in cmd and not under Cygwin/MinGW bash
// otherwise the CTRL-C handling will not work properly

extern crate win_sig;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use win_sig::{HandleOutcome, Signal};

enum RunError {
    Unknown,
}

impl From<()> for RunError {
    fn from(_: ()) -> RunError {
        RunError::Unknown
    }
}

fn run() -> Result<(), RunError> {
    const EXIT_COUNT: usize = 5;

    win_sig::reset()?;

    let counter = Arc::new(AtomicUsize::new(0));
    let handler_counter = counter.clone();

    win_sig::set_handler(move |sig| {
        match sig {
            Signal::CtrlCEvent => {
                handler_counter.fetch_add(1, Ordering::SeqCst);
                let left_count = EXIT_COUNT - handler_counter.load(Ordering::Relaxed);

                if left_count > 0 {
                    println!(
                        "CTRL-C event captured! Press {} more time(s) to terminate!",
                        left_count
                    );
                    HandleOutcome::Handled
                } else {
                    // passthrough to allow the OS CTRL-C to kill the loop
                    println!("CTRL-C event captured! Terminating program...");
                    HandleOutcome::Passthrough
                }
            }
            _ => {
                println!("Other event captured!");
                HandleOutcome::Passthrough
            }
        }
    })?;

    println!("Program busy looping, pressing CTRL-C for effect...");
    loop {}
}

fn main() {
    match run() {
        Ok(()) => println!("Program completed!"),
        Err(_) => println!("Unknown error in program, terminating..."),
    }
}
