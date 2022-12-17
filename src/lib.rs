//! Zero Shell library
//!
//! # Threads
//!
//! - `main` reads user input and sends it to the `worker` thread.
//! - `signal_handler` receieves signals and sends them to the `worker` thread.
//! - `worker` is the core processor and manages processes.

pub(crate) mod util;

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

