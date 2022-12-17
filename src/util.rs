//! Utilities

/// Calls a syscall function while handling the EINTR signal.
pub fn run_syscall<T>(f: impl Fn() -> Result<T, nix::Error>) -> Result<T, nix::Error> {
    loop {
        match f() {
            // EINTR isgnal indicates that some interruption happened while calling the syscall.
            // Retry the syscall:
            Err(nix::Error::EINTR) => continue,
            res => break res,
        }
    }
}
