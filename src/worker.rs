//! Worker thread.

use nix::unistd;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::mpsc,
    thread,
};

use crate::{ShellMsg, WorkerMsg};

#[derive(Debug)]
pub struct Worker {
    /// Exit code
    exit_code: i32,

    /// Foreground process ID
    fg: Option<unistd::Pid>,
    // jobs: BTreeMap<usize, (unistd::Pid, String)>,
    // gpid_to_pid: HashMap<unistd::Pid, (usize, HashSet<unistd::Pid>)>,
    // pid_to_info: HashMap<unistd::Pid, ProcessInfo>,
}

impl Worker {
    fn new() -> Self {
        Self {
            exit_code: 0,
            // the shell is the foreground process
            fg: None,
        }
    }
}

pub(crate) fn spawn_worker(worker_rx: mpsc::Receiver<WorkerMsg>, shell_tx: mpsc::SyncSender<ShellMsg>) {
    let mut worker = Worker::new();

    thread::spawn(move || {
        for msg in worker_rx.iter() {
            match msg {
                _ => todo!(),
            }
        }
    });
}
