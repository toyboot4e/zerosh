//! Zero Shell library
//!
//! # Threads
//!
//! - `main` reads user input and sends it to the `worker` thread.
//! - `signal_handler` receieves signals and sends them to the `worker` thread.
//! - `worker` is the core processor and manages processes.

pub mod shell;

pub(crate) mod util;

mod worker;

use nix::sys::signal;

use std::{sync::mpsc, thread};

pub type DynError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Message to the `worker` thread
#[derive(Debug, Clone)]
enum WorkerMsg {
    /// Message from the `signal_handler` thread, i.e., a signal.
    Signal { signal: i32 },

    /// Message from the `main` thread, i.e., user input.
    Cmd { cmd: String },
}

/// Message to the `main` thread
#[derive(Debug, Clone)]
enum ShellMsg {
    /// Continue reading user input
    Continue { code: i32 },

    /// Quit the shell
    Quit { code: i32 },
}

#[derive(Debug)]
pub struct Shell {
    log_file: String,
}

impl Shell {
    pub fn new(log_file: String) -> Self {
        Self { log_file }
    }
}

/// Creates `worker` and the `signal_handler` threads from the `main` thread and starts handling
/// user input.
pub fn run_shell(sh: &Shell) -> Result<(), DynError> {
    unsafe {
        signal::signal(signal::Signal::SIGTTOU, signal::SigHandler::SigIgn).unwrap();
    }

    let mut editor = rustyline::Editor::<()>::new()?;

    if let Err(err) = editor.load_history(&sh.log_file) {
        eprintln!("unable to read history file: {err}");
    }

    // - tx: transmitter (sender)
    // - rx: receiver
    let (worker_tx, worker_rx) = mpsc::channel();
    let (shell_tx, shell_rx) = mpsc::sync_channel(0);

    self::spawn_signal_handler(worker_tx.clone())?;
    crate::worker::spawn_worker(worker_rx, shell_tx);

    loop {
        todo!();
    }

    Ok(())
}

/// Spawns the `signal_handler` threasd
fn spawn_signal_handler(tx: mpsc::Sender<WorkerMsg>) -> Result<(), DynError> {
    let mut signals = signal_hook::iterator::Signals::new({
        use signal_hook::consts::*;
        &[SIGINT, SIGTSTP, SIGCHLD]
    })?;

    thread::spawn(move || {
        for signal in signals.forever() {
            tx.send(WorkerMsg::Signal { signal }).unwrap();
        }
    });

    Ok(())
}
