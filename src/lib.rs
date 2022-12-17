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

    let (worker_tx, worker_rx) = mpsc::channel();
    let (shell_tx, mut shell_rx) = mpsc::sync_channel(0);

    let mut state = State::create(&sh.log_file, worker_tx.clone())?;

    self::spawn_signal_handler(worker_tx.clone())?;
    crate::worker::spawn_worker(worker_rx, shell_tx.clone());

    loop {
        if !state.process(sh, &mut shell_rx)? {
            break;
        }
    }

    Ok(())
}

/// Spawns the `signal_handler` thread
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

#[derive(Debug)]
struct State {
    editor: rustyline::Editor<()>,
    worker_tx: mpsc::Sender<WorkerMsg>,
    exit_code: i32,
    last_exit_code: i32,
}

impl State {
    fn create(log_file: &str, worker_tx: mpsc::Sender<WorkerMsg>) -> rustyline::Result<Self> {
        let mut editor = rustyline::Editor::<()>::new()?;

        if let Err(err) = editor.load_history(log_file) {
            eprintln!("unable to read history file: {err}");
        }

        Ok(Self {
            editor,
            worker_tx,
            exit_code: 0,
            last_exit_code: 0,
        })
    }

    fn prompt(&self) -> String {
        let face = if self.last_exit_code == 0 {
            '\u{1F642}'
        } else {
            '\u{1F480}'
        };

        format!("ZeroSh {face} %>")
    }

    /// Returns `Ok(true)` if the shell can continue processing
    fn process(
        &mut self,
        sh: &Shell,
        shell_rx: &mut mpsc::Receiver<ShellMsg>,
    ) -> Result<bool, DynError> {
        let prompt = self.prompt();

        // TODO: Allow multiline input (?)
        use rustyline::error::ReadlineError;
        let line = match self.editor.readline(&prompt) {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                todo!()
            }
            Err(ReadlineError::Eof) => {
                todo!()
            }
            Err(err) => {
                eprintln!("ZeroSh: read error\n{err}");
                self.exit_code = 1;
                return Ok(false);
            }
        };

        let line = line.trim();
        if line.is_empty() {
            return Ok(true);
        }
        self.editor.add_history_entry(line);

        self.worker_tx.send(WorkerMsg::Cmd {
            cmd: line.to_string(),
        })?;

        match shell_rx.recv()? {
            ShellMsg::Continue { code } => Ok(self.last_exit_code == code),
            ShellMsg::Quit { code } => {
                self.exit_code = code;
                Ok(false)
            }
        }
    }
}
