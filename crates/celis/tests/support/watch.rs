//! A `celis` command that runs on a terminal, and what it prints there.

use super::Repo;
use std::cell::Cell;
use std::io::Read;
use std::os::fd::FromRawFd;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};

/// A running `celis tree` on a terminal, the text it printed so far, and the terminal's rows once set.
pub struct Watch {
    pub child: Child,
    lead: i32,
    bytes: Receiver<Vec<u8>>,
    pub seen: Vec<u8>,
    height: Cell<Option<u16>>,
}

impl Watch {
    /// Runs `celis <args>` on a new terminal in the main checkout of `repo`, at about noon.
    pub fn start(repo: &Repo, args: &[&str]) -> Watch {
        let (mut lead, mut follow) = (0, 0);
        let opened = unsafe {
            libc::openpty(
                &mut lead,
                &mut follow,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(opened, 0, "openpty");
        let side = |fd| unsafe { Stdio::from_raw_fd(libc::dup(fd)) };
        let child = Command::new(env!("CARGO_BIN_EXE_celis"))
            .args(args)
            .current_dir(repo.path())
            .env_clear()
            .env("PATH", path(repo))
            .env("CELIS_HOME", repo.home())
            .env("TZ", super::noon())
            .stdin(side(follow))
            .stdout(side(follow))
            .stderr(side(follow))
            .spawn()
            .unwrap();
        unsafe { libc::close(follow) };
        let (send, bytes) = channel();
        let mut terminal = unsafe { std::fs::File::from_raw_fd(libc::dup(lead)) };
        std::thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            while let Ok(n) = terminal.read(&mut buffer) {
                if n == 0 || send.send(buffer[..n].to_vec()).is_err() {
                    break;
                }
            }
        });
        Watch {
            child,
            lead,
            bytes,
            seen: Vec::new(),
            height: Cell::new(None),
        }
    }

    /// All the text printed so far, after a wait of `wait` for more.
    pub fn shown(&mut self, wait: Duration) -> String {
        let end = Instant::now() + wait;
        while let Ok(chunk) = self
            .bytes
            .recv_timeout(end.saturating_duration_since(Instant::now()))
        {
            self.seen.extend(chunk);
        }
        plain(&String::from_utf8_lossy(&self.seen))
    }

    /// The trees printed so far, one string each with no line or frame codes, after a wait of `wait`
    /// for more.
    pub fn trees(&mut self, wait: Duration) -> Vec<String> {
        self.screens(wait).iter().map(|t| plain(t)).collect()
    }

    /// The screen after each frame printed so far, with its colour codes: its rows above the last joined
    /// by `\r\n`, then the last after `\r\n`, or after a move to it once the terminal's rows are set.
    fn screens(&mut self, wait: Duration) -> Vec<String> {
        self.shown(wait);
        let text = String::from_utf8_lossy(&self.seen).into_owned();
        let (mut rows, mut at, mut screens) = (Vec::<String>::new(), 0, Vec::new());
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            let row = |rows: &mut Vec<String>, at: usize| {
                if rows.len() <= at {
                    rows.resize(at + 1, String::new());
                }
            };
            match c {
                '\x1b' if chars.peek() == Some(&'[') => {
                    chars.next();
                    let mut code = String::new();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        code.push(c);
                        if c.is_ascii_alphabetic() {
                            break;
                        }
                    }
                    match code.chars().last() {
                        Some('H') => {
                            at = code
                                .split(';')
                                .next()
                                .and_then(|r| r.parse().ok())
                                .unwrap_or(1)
                                - 1;
                            row(&mut rows, at);
                            rows[at].clear();
                        }
                        Some('J') => rows.truncate(at),
                        Some('m') => {
                            row(&mut rows, at);
                            rows[at].push_str(&format!("\x1b[{code}"));
                        }
                        _ if code == "?2026l" => screens.push(self.screen(&rows)),
                        _ => {}
                    }
                }
                '\r' => {}
                '\n' => {
                    at += 1;
                    row(&mut rows, at);
                    rows[at].clear();
                }
                c => {
                    row(&mut rows, at);
                    rows[at].push(c);
                }
            }
        }
        screens
    }

    /// The screen `rows` show, as `screens` gives it, cut to the terminal's rows once set.
    fn screen(&self, rows: &[String]) -> String {
        let shown = self
            .height
            .get()
            .map_or(rows.len(), |h| rows.len().min(h.into()));
        let rows = &rows[..shown];
        let Some((last, body)) = rows.split_last() else {
            return String::new();
        };
        match self.height.get() {
            Some(height) if rows.len() == usize::from(height) => {
                let end = body
                    .iter()
                    .rposition(|r| !r.is_empty())
                    .map_or(0, |n| n + 1);
                format!("{}\x1b[{height};1H{last}", body[..end].join("\r\n"))
            }
            _ => format!("{}\r\n{last}", body.join("\r\n")),
        }
    }

    /// Waits until the newest tree holds `text`; the time it took.
    pub fn until(&mut self, text: &str) -> Duration {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if self
                .trees(Duration::from_millis(50))
                .last()
                .is_some_and(|t| t.contains(text))
            {
                return start.elapsed();
            }
        }
        panic!("no {text:?} in {:?}", self.shown(Duration::ZERO));
    }

    /// The newest tree with its colour codes, after a wait of `wait` for more.
    pub fn coloured(&mut self, wait: Duration) -> String {
        self.screens(wait).pop().unwrap_or_default()
    }

    /// The lines of the newest tree, each with no codes, above the key line.
    pub fn rows(&mut self) -> Vec<String> {
        let last = self.trees(Duration::ZERO).pop().unwrap_or_default();
        let body = last.split("\x1b[").next().unwrap_or("").to_string();
        body.split("\r\n").map(str::to_string).collect()
    }

    /// Waits until the newest tree no longer holds `text`.
    pub fn without(&mut self, text: &str) {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if self
                .trees(Duration::from_millis(50))
                .last()
                .is_some_and(|t| !t.contains(text))
            {
                return;
            }
        }
        panic!("still {text:?} in {:?}", self.shown(Duration::ZERO));
    }

    /// Types `keys` on the terminal.
    pub fn press(&self, keys: &str) {
        let written = unsafe { libc::write(self.lead, keys.as_ptr().cast(), keys.len()) };
        assert_eq!(written, keys.len() as isize);
    }

    /// Turns the terminal `columns` wide, and signals it.
    pub fn resize(&self, columns: u16) {
        self.size(24, columns);
    }

    /// Turns the terminal `rows` high and `columns` wide, and signals it.
    pub fn size(&self, rows: u16, columns: u16) {
        self.height.set(Some(rows));
        let size = libc::winsize {
            ws_row: rows,
            ws_col: columns,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        assert_eq!(
            unsafe { libc::ioctl(self.lead, libc::TIOCSWINSZ, &size) },
            0
        );
        let pid = libc::pid_t::try_from(self.child.id()).unwrap();
        assert_eq!(unsafe { libc::kill(pid, libc::SIGWINCH) }, 0);
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        unsafe { libc::close(self.lead) };
    }
}

/// The watch's `PATH`: the home's `bin` folder, then the test's own.
fn path(repo: &Repo) -> String {
    let own = std::env::var("PATH").unwrap_or_default();
    format!("{}:{own}", repo.home().join("bin").display())
}

/// `text` with no colour codes.
pub fn plain(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("\x1b[") {
        out.push_str(&rest[..at]);
        let code = &rest[at + 2..];
        match code.find(|c: char| c.is_ascii_alphabetic()) {
            Some(end) if code.as_bytes()[end] == b'm' => rest = &code[end + 1..],
            _ => {
                out.push_str("\x1b[");
                rest = code;
            }
        }
    }
    out + rest
}
