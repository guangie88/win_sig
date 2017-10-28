# `win_sig`
Wraps Windows signal handling mechanism with a safer and easier to understand callback API. Only works for Windows.

[![Build status](https://ci.appveyor.com/api/projects/status/0pl4g0tb909u3j6s/branch/master?svg=true)](https://ci.appveyor.com/project/guangie88/win-sig/branch/master)

May contain bugs at this stage.

## How to Build
`cargo build --examples` for Debug build, `cargo build --release --examples` for Release build. Both commands will build the library as well as the executable example to demonstrate the use of the library.

## How to Run CTRL-C Example
The `five_ctrl_c` example cannot be executed under Cygwin/MinGW `bash` or `cargo run` context. As such, only `cmd` should be used and directly run `target\debug\examples\five_ctrl_c.exe` for Debug build, or `target\release\examples\five_ctrl_c.exe` for Release build.

## CTRL-C Handling Example (`five_ctrl_c.rs`)
```Rust
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
```