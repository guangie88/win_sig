// executable must be run in cmd without cargo run and not under Cygwin/MinGW bash
// otherwise the CTRL-C handling will not work properly

extern crate win_sig;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use win_sig::{CtrlEvent, HandleError, HandleOutcome, HandleResult};

fn run() -> HandleResult {
    const EXIT_COUNT: usize = 5;

    win_sig::reset()?;

    let counter = Arc::new(AtomicUsize::new(0));
    let handler_counter = counter.clone();

    win_sig::set_handler(move |sig| {
        match sig {
            CtrlEvent::C => {
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
        Err(HandleError::Lock) => println!("Encountered handler mutex lock error, terminating..."),
        Err(HandleError::Os) => println!("Unable to set handler correctly, terminating..."),
    }
}
