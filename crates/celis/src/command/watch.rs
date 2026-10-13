//! `celis tree` on a terminal: the tree, printed again each time the store changes.

use super::tree::EMPTY;
use super::{Args, Command};
use celis_trails::Failure;
use notify::{RecursiveMode, Watcher};
use std::io::Read;
use std::io::Write;
use std::os::fd::FromRawFd;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::time::Duration;

impl Command {
    /// Prints the tree `args` names, then prints it again when a record lands, a time it shows changes, or
    /// the terminal's width changes, and sleeps between. With no store, it writes nothing and waits for
    /// one. Runs until it is stopped.
    pub(super) fn watch(&self, args: &Args, out: &mut dyn Write) -> Result<(), Failure> {
        let mut shown = String::new();
        if !self.held()? {
            if let Some(id) = args.words.get(1) {
                return Err(Failure::no_step(id));
            }
            show(out, &format!("{EMPTY}\n"), &mut shown)?;
            while !self.held()? {
                std::thread::sleep(Duration::from_millis(250));
            }
        }
        let mut trails = self.trails()?;
        if let Some(id) = args.words.get(1) {
            trails.steps().find(id)?;
        }
        let (send, events) = channel();
        resizes(send.clone());
        let landed = move |event: notify::Result<notify::Event>| {
            if event.is_ok_and(|e| !e.kind.is_access()) {
                let _ = send.send(Wake::Record);
            }
        };
        let mut watcher = notify::recommended_watcher(landed).map_err(anyhow::Error::from)?;
        watcher
            .watch(
                &self.home()?.join("trails/shared"),
                RecursiveMode::NonRecursive,
            )
            .map_err(anyhow::Error::from)?;
        let mut width = self.columns();
        loop {
            trails.now = super::now();
            let (mut tree, next) = self.frame(&trails, args, width)?;
            if tree.is_empty() && args.words.len() < 2 {
                tree = format!("{EMPTY}\n");
            }
            show(out, &tree, &mut shown)?;
            let wake = match next {
                Some(at) => events.recv_timeout(until(at)),
                None => events.recv().map_err(|_| RecvTimeoutError::Disconnected),
            };
            match wake {
                Ok(Wake::Record) => {
                    while events.recv_timeout(Duration::from_millis(50)).is_ok() {}
                    trails = self.trails()?;
                }
                Ok(Wake::Resize) => {}
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => return Ok(()),
            }
            if self.columns() != width {
                width = self.columns();
                shown.clear();
            }
        }
    }
}

/// Prints `tree` from the top left of a cleared screen, when it is not empty and not what `shown` holds.
fn show(out: &mut dyn Write, tree: &str, shown: &mut String) -> Result<(), Failure> {
    if tree.is_empty() || tree == shown {
        return Ok(());
    }
    write!(out, "\x1b[H\x1b[2J\x1b[3J{tree}")
        .and_then(|_| out.flush())
        .map_err(anyhow::Error::from)?;
    *shown = tree.to_string();
    Ok(())
}

/// The time from now until the second `at` since the epoch starts.
fn until(at: u64) -> Duration {
    (std::time::UNIX_EPOCH + Duration::from_secs(at))
        .duration_since(std::time::SystemTime::now())
        .unwrap_or_default()
}

/// What wakes the watch before its next tick.
enum Wake {
    Record,
    Resize,
}

/// The pipe the resize signal writes to; -1 before `resizes` opens it.
static RESIZED: AtomicI32 = AtomicI32::new(-1);

/// Sends a `Resize` to `send` each time the terminal's size changes, the moment the signal arrives.
fn resizes(send: Sender<Wake>) {
    let mut fds = [0; 2];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } != 0 {
        return;
    }
    RESIZED.store(fds[1], Ordering::Relaxed);
    let handler = resized as extern "C" fn(libc::c_int);
    unsafe { libc::signal(libc::SIGWINCH, handler as libc::sighandler_t) };
    let mut pipe = unsafe { std::fs::File::from_raw_fd(fds[0]) };
    std::thread::spawn(move || {
        let mut byte = [0u8; 1];
        while pipe.read(&mut byte).is_ok_and(|n| n > 0) && send.send(Wake::Resize).is_ok() {}
    });
}

/// Writes one byte to the resize pipe; it does only what a signal handler may.
extern "C" fn resized(_: libc::c_int) {
    let fd = RESIZED.load(Ordering::Relaxed);
    unsafe { libc::write(fd, [0u8].as_ptr().cast(), 1) };
}
