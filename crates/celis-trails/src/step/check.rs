//! A step's check: a shell command whose exit says whether the work holds.

use crate::Refusal;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::time::{Duration, Instant};

/// A shell command, run in a folder and stopped after a number of seconds.
pub struct Check<'a> {
    command: &'a str,
    folder: &'a Path,
    limit: u64,
}

impl<'a> Check<'a> {
    pub fn new(command: &'a str, folder: &'a Path, limit: u64) -> Check<'a> {
        Check {
            command,
            folder,
            limit,
        }
    }

    /// Runs the command; `check-failed` with the last 20 lines of its output when it fails, or when it
    /// runs past the limit.
    pub fn run(&self) -> Result<(), Refusal> {
        let failed = |e: std::io::Error| Refusal::CheckFailed(e.to_string());
        let log = std::env::temp_dir().join(format!("celis-check-{}", std::process::id()));
        let file = std::fs::File::create(&log).map_err(failed)?;
        let mut child = std::process::Command::new("sh")
            .args(["-c", self.command])
            .current_dir(self.folder)
            .stdout(file.try_clone().map_err(failed)?)
            .stderr(file)
            .process_group(0)
            .spawn()
            .map_err(failed)?;
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().map_err(failed)? {
                break Some(status);
            }
            if started.elapsed().as_secs() >= self.limit {
                let group = libc::pid_t::try_from(child.id()).unwrap_or(0);
                unsafe { libc::kill(-group, libc::SIGKILL) };
                let _ = child.wait();
                break None;
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        let printed = std::fs::read_to_string(&log).unwrap_or_default();
        let _ = std::fs::remove_file(&log);
        let lines: Vec<&str> = printed.lines().collect();
        let end = lines[lines.len().saturating_sub(20)..].join("\n");
        match status {
            Some(status) if status.success() => Ok(()),
            Some(_) => Err(Refusal::CheckFailed(end)),
            None => Err(Refusal::CheckFailed(format!(
                "{end}\nthe check ran past {} seconds",
                self.limit
            ))),
        }
    }
}
