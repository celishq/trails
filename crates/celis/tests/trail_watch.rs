//! CIP-14, "Watching".

mod support;

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use support::{add, cli, err, refusal, Repo};

/// A running `celis watch`, and the lines it printed so far.
struct Watch {
    child: Child,
    lines: Receiver<String>,
    seen: Vec<String>,
}

impl Watch {
    fn start(repo: &Repo, args: &[&str]) -> Watch {
        let mut child = Command::new(env!("CARGO_BIN_EXE_celis"))
            .arg("watch")
            .args(args)
            .current_dir(repo.path())
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .env("CELIS_HOME", repo.home())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let (send, lines) = channel();
        let stdout = child.stdout.take().unwrap();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let _ = send.send(line);
            }
        });
        Watch {
            child,
            lines,
            seen: Vec::new(),
        }
    }

    /// The trees printed so far, one string each.
    fn trees(&mut self, wait: Duration) -> Vec<String> {
        let end = Instant::now() + wait;
        while let Ok(line) = self
            .lines
            .recv_timeout(end.saturating_duration_since(Instant::now()))
        {
            self.seen.push(line);
        }
        self.seen
            .split(String::is_empty)
            .filter(|t| !t.is_empty())
            .map(|t| t.join("\n"))
            .collect()
    }

    /// Waits until the newest tree holds `text`; the time it took.
    fn until(&mut self, text: &str) -> Duration {
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
        panic!("no tree holds {text:?}: {:?}", self.seen);
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn records(repo: &Repo) -> usize {
    std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .count()
}

/// `watch <id>` prints the tree, then prints it again within a second after a record lands, only when
/// the tree changed, with an empty line between two trees off a terminal, and writes no record
/// (`spec/trails/14-trails.md`, "Watching").
#[test]
fn watch_prints_the_tree_again_when_it_changes() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("the trail");
    let quiet = records(&repo);
    assert_eq!(watch.trees(Duration::from_millis(1200)).len(), 1);
    assert_eq!(records(&repo), quiet);

    let step = add(&repo, &["--under", &trail[..8], "--", "the step"]);
    let took = watch.until(&format!("○ {}", &step[..8]));
    assert!(took < Duration::from_millis(1500), "{took:?}");
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    watch.until(&format!("◐ {}", &step[..8]));
    let shown = watch.trees(Duration::from_millis(200)).len();

    let o = cli(
        &repo,
        &["decide", "--", "When you add a page: copy the frame."],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(watch.trees(Duration::from_millis(1500)).len(), shown);
}

/// `watch` with no id prints nothing while no trail is active, then the newest active trail, and
/// moves to a newer trail when one starts (`spec/trails/14-trails.md`, "Watching").
#[test]
fn watch_with_no_id_follows_the_newest_active_trail() {
    let repo = Repo::new();
    let mut watch = Watch::start(&repo, &[]);
    assert!(
        watch.trees(Duration::from_millis(800)).is_empty(),
        "{:?}",
        watch.seen
    );
    let first = add(&repo, &["--", "the first trail"]);
    watch.until(&first[..8]);
    std::thread::sleep(Duration::from_millis(1100));
    let second = add(&repo, &["--", "the second trail"]);
    watch.until(&second[..8]);
    let newest = watch.trees(Duration::from_millis(200)).pop().unwrap();
    assert!(!newest.contains(&first[..8]), "{newest}");
}

/// `watch` refuses an id no step has, with `no-step` (`spec/trails/14-trails.md`, "Watching").
#[test]
fn watch_refuses_an_unknown_id() {
    let repo = Repo::new();
    add(&repo, &["--", "a trail"]);
    refusal(&cli(&repo, &["watch", "deadbeef", "--json"]), 1, "no-step");
}

/// The text `celis watch <args>` writes to a terminal, read until it holds each of `texts` in turn; after
/// the first, the terminal turns `resize` columns wide and signals it, when given.
fn on_terminal(repo: &Repo, args: &[&str], texts: &[&str], resize: Option<u16>) -> String {
    use std::io::Read;
    use std::os::fd::FromRawFd;
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
    let mut child = Command::new(env!("CARGO_BIN_EXE_celis"))
        .arg("watch")
        .args(args)
        .current_dir(repo.path())
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("CELIS_HOME", repo.home())
        .stdin(side(follow))
        .stdout(side(follow))
        .stderr(side(follow))
        .spawn()
        .unwrap();
    unsafe { libc::close(follow) };
    let (send, bytes) = channel();
    let mut terminal = unsafe { std::fs::File::from_raw_fd(lead) };
    std::thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        while let Ok(n) = terminal.read(&mut buffer) {
            if n == 0 || send.send(buffer[..n].to_vec()).is_err() {
                break;
            }
        }
    });
    let mut seen = Vec::new();
    let wait_for = |text: &str, seen: &mut Vec<u8>| {
        let end = Instant::now() + Duration::from_secs(5);
        while !String::from_utf8_lossy(seen).contains(text) {
            let left = end.saturating_duration_since(Instant::now());
            match bytes.recv_timeout(left) {
                Ok(chunk) => seen.extend(chunk),
                Err(_) => panic!("no {text:?} in {:?}", String::from_utf8_lossy(seen)),
            }
        }
    };
    for (n, text) in texts.iter().enumerate() {
        wait_for(text, &mut seen);
        if let (0, Some(columns)) = (n, resize) {
            let size = libc::winsize {
                ws_row: 24,
                ws_col: columns,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };
            assert_eq!(unsafe { libc::ioctl(lead, libc::TIOCSWINSZ, &size) }, 0);
            let pid = libc::pid_t::try_from(child.id()).unwrap();
            assert_eq!(unsafe { libc::kill(pid, libc::SIGWINCH) }, 0);
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    String::from_utf8_lossy(&seen).into_owned()
}

/// On a terminal, `watch` clears the screen and its scrollback, and prints each tree from the top left,
/// with no empty line between two trees (`spec/trails/14-trails.md`, "Watching").
#[test]
fn watch_on_a_terminal_clears_the_screen_before_each_tree() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let adder = std::thread::spawn({
        let (home, path, trail) = (repo.home(), repo.path(), trail.clone());
        move || {
            std::thread::sleep(Duration::from_millis(700));
            let o = Command::new(env!("CARGO_BIN_EXE_celis"))
                .args(["step", "add", "--under", &trail[..8], "--", "the step"])
                .current_dir(path)
                .env("CELIS_HOME", home)
                .output()
                .unwrap();
            assert!(o.status.success());
        }
    });
    let shown = on_terminal(&repo, &[&trail[..8]], &["the trail", "the step"], None);
    adder.join().unwrap();
    let clear = "\x1b[H\x1b[2J\x1b[3J";
    let trees: Vec<&str> = shown.split(clear).skip(1).collect();
    assert!(trees.len() >= 2, "{shown:?}");
    assert!(shown.starts_with(clear), "{shown:?}");
    assert!(!trees[0].contains("the step"), "{shown:?}");
    assert!(trees[trees.len() - 1].contains("the step"), "{shown:?}");
    assert!(!shown.contains("\r\n\r\n"), "{shown:?}");
}

/// On a terminal, `watch` prints a tree whose times stand still again, cut to the new width, when the
/// terminal's width changes (`spec/trails/14-trails.md`, "Watching").
#[test]
fn watch_on_a_terminal_prints_the_tree_again_when_its_width_changes() {
    let repo = Repo::new();
    let hours = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 7_200;
    let text = "a trail whose words run well past thirty characters";
    let trail = support::step_note(&repo, &support::account(), None, text, hours).id;
    let shown = on_terminal(&repo, &[&trail[..8]], &["thirty characters", "…"], Some(30));
    let clear = "\x1b[H\x1b[2J\x1b[3J";
    let trees: Vec<&str> = shown.split(clear).skip(1).collect();
    assert!(trees.len() >= 2, "{shown:?}");
    assert!(
        !trees[trees.len() - 1].contains("thirty characters"),
        "{shown:?}"
    );
}

/// On a terminal, `watch` prints the tree again as the times it shows change, with no record landing
/// (`spec/trails/14-trails.md`, "Watching").
#[test]
fn watch_on_a_terminal_prints_the_tree_again_as_its_times_change() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let shown = on_terminal(&repo, &[&trail[..8]], &["the trail", "3s"], None);
    let clear = "\x1b[H\x1b[2J\x1b[3J";
    assert!(shown.split(clear).skip(1).count() >= 2, "{shown:?}");
}
