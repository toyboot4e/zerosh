//! Zero Shell library
//!
//! # Threads
//!
//! - `main` reads user input and sends it to the `worker` thread.
//! - `signal_handler` receieves signals and sends them to the `worker` thread.
//! - `worker` is the core processor and the process manager.

pub mod shell;

pub(crate) mod util;

mod worker;

use nix::sys::signal;

use std::{ops::ControlFlow, sync::mpsc, thread};

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

/// Creates the `worker` and the `signal_handler` threads from the `main` thread and starts handling
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
        if self::process(&mut state, sh, &mut shell_rx)?.is_break() {
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
}

/// Returns `Ok(true)` if the shell can continue processing
fn process(
    state: &mut State,
    sh: &Shell,
    shell_rx: &mut mpsc::Receiver<ShellMsg>,
) -> Result<ControlFlow<()>, DynError> {
    let prompt = state.prompt();

    // TODO: Allow multiline input (?)
    use rustyline::error::ReadlineError;
    use ControlFlow::*;

    let line = match state.editor.readline(&prompt) {
        Ok(line) => line,
        Err(ReadlineError::Interrupted) => {
            eprintln!("ZeroSh: you can exit with `Ctrl+d`");
            return Ok(Continue(()));
        }
        Err(ReadlineError::Eof) => {
            state.worker_tx.send(WorkerMsg::Cmd {
                cmd: "exit".to_string(),
            })?;

            match shell_rx.recv()? {
                ShellMsg::Quit { code } => {
                    state.exit_code = code;
                    return Ok(Break(()));
                }
                _ => panic!("failed to exit")
            }
        }
        Err(err) => {
            eprintln!("ZeroSh: read error\n{err}");
            state.exit_code = 1;
            return Ok(Break(()));
        }
    };

    let line = line.trim();
    if line.is_empty() {
        return Ok(Continue(()));
    }
    state.editor.add_history_entry(line);

    state.worker_tx.send(WorkerMsg::Cmd {
        cmd: line.to_string(),
    })?;

    match shell_rx.recv()? {
        ShellMsg::Continue { code } => {
            state.last_exit_code = code;
            Ok(Continue(()))
        }
        ShellMsg::Quit { code } => {
            state.exit_code = code;
            Ok(Break(()))
        }
    }
}
