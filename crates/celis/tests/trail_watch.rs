//! CIP-02, "Watching".

mod support;

use std::io::Read;
use std::os::fd::FromRawFd;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use support::{add, cli, err, finish, refusal, Repo};

/// What starts each tree: a frame drawn from the top left.
const FRAME: &str = "\x1b[?2026h\x1b[H";

/// The spinner's frames.
const SPIN: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

/// A running `celis tree` on a terminal, and the text it printed so far.
struct Watch {
    child: Child,
    lead: i32,
    bytes: Receiver<Vec<u8>>,
    seen: Vec<u8>,
}

impl Watch {
    fn start(repo: &Repo, args: &[&str]) -> Watch {
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
            .arg("tree")
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
        }
    }

    /// All the text printed so far, after a wait of `wait` for more.
    fn shown(&mut self, wait: Duration) -> String {
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
    fn trees(&mut self, wait: Duration) -> Vec<String> {
        self.shown(wait)
            .split(FRAME)
            .skip(1)
            .map(|t| {
                t.replace("\x1b[K", "")
                    .replace("\x1b[J", "")
                    .replace("\x1b[?2026l", "")
            })
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
        panic!("no {text:?} in {:?}", self.shown(Duration::ZERO));
    }

    /// Types `keys` on the terminal.
    fn press(&self, keys: &str) {
        let written = unsafe { libc::write(self.lead, keys.as_ptr().cast(), keys.len()) };
        assert_eq!(written, keys.len() as isize);
    }

    /// Turns the terminal `columns` wide, and signals it.
    fn resize(&self, columns: u16) {
        self.size(24, columns);
    }

    /// Turns the terminal `rows` high and `columns` wide, and signals it.
    fn size(&self, rows: u16, columns: u16) {
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

/// `text` with no colour codes.
fn plain(text: &str) -> String {
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

fn records(repo: &Repo) -> usize {
    std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .count()
}

/// On a terminal, `tree <id>` prints the tree, then prints it again within a second after a record lands,
/// and writes no record (`spec/02-cli.md`, "Watching").
#[test]
fn watch_prints_the_tree_again_when_a_record_lands() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("the trail");
    let quiet = records(&repo);
    watch.trees(Duration::from_millis(1200));
    assert_eq!(records(&repo), quiet);

    let step = add(&repo, &["--under", &trail[..8], "--", "the step"]);
    let took = watch.until(&step[..8]);
    assert!(took < Duration::from_millis(1500), "{took:?}");
}

/// On a terminal, `tree <id>` of a trail whose times stand still prints it again only when a record
/// changes the tree (`spec/02-cli.md`, "Watching").
#[test]
fn watch_prints_the_tree_again_only_when_it_changes() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    finish(&repo, &trail[..8]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("✓ the trail");
    let shown = watch.trees(Duration::from_millis(1200)).len();
    let o = cli(
        &repo,
        &["decide", "--", "When you add a page: copy the frame."],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(watch.trees(Duration::from_millis(1500)).len(), shown);
}

/// On a terminal, `tree` with no id says no trail is there and names `tree --all`, then prints what `tree`
/// prints: the trails of every project (`spec/02-cli.md`, "Watching").
#[test]
fn watch_with_no_id_prints_what_tree_prints() {
    let repo = Repo::new();
    let mut watch = Watch::start(&repo, &[]);
    watch.until("No active trails");
    watch.until("celis tree --all");
    let first = add(&repo, &["--", "the first trail"]);
    watch.until(&first[..8]);
    let shop = repo.other("shop");
    let o = repo.run(&shop, &["step", "add", "--", "the shop trail"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    watch.until("the shop trail");
    finish(&repo, &first[..8]);
    watch.until("✓ the first trail");
}

/// On a home with no store and no phrase, `tree` on a terminal shows the empty state, writes nothing, and
/// shows the first trail once another command writes it (`spec/01-decisions.md`, "The store").
#[test]
fn watch_on_an_empty_home_writes_nothing_and_waits_for_the_store() {
    let repo = Repo::new();
    std::fs::remove_file(repo.home().join("phrase")).unwrap();
    let mut watch = Watch::start(&repo, &[]);
    watch.until("No active trails");
    assert!(!repo.home().join("phrase").exists());
    assert!(!repo.store().exists());
    let trail = add(&repo, &["--", "the first trail"]);
    watch.until(&trail[..8]);
}

/// On a terminal, `tree` refuses an id no step has (`spec/02-cli.md`, "Watching").
#[test]
fn watch_refuses_an_unknown_id() {
    let repo = Repo::new();
    add(&repo, &["--", "a trail"]);
    let mut watch = Watch::start(&repo, &["deadbeef"]);
    assert_eq!(watch.child.wait().unwrap().code(), Some(1));
    assert!(
        watch
            .shown(Duration::from_millis(200))
            .contains("no step has the id deadbeef"),
        "{:?}",
        watch.shown(Duration::ZERO)
    );
}

/// `tree` takes no `--watch`: it is a usage error (`spec/02-cli.md`, "Watching").
#[test]
fn tree_refuses_watch() {
    let repo = Repo::new();
    refusal(&cli(&repo, &["tree", "--watch", "--json"]), 2, "usage");
}

/// On a terminal, `tree` draws on the alternate screen, each tree from the top left over the last one, with
/// no clear (`spec/02-cli.md`, "Watching").
#[test]
fn watch_draws_each_tree_over_the_last_on_the_alternate_screen() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("the trail");
    add(&repo, &["--under", &trail[..8], "--", "the step"]);
    watch.until("the step");
    let shown = watch.shown(Duration::ZERO);
    let trees = watch.trees(Duration::ZERO);
    assert!(trees.len() >= 2, "{shown:?}");
    assert!(shown.starts_with("\x1b[?1049h"), "{shown:?}");
    assert!(!trees[0].contains("the step"), "{shown:?}");
    assert!(!shown.contains("\x1b[2J"), "{shown:?}");
}

/// On a terminal, `tree` prints a tree whose times stand still again, wrapped to the new width, when the
/// terminal's width changes (`spec/02-cli.md`, "Watching").
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
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("thirty characters");
    watch.resize(30);
    watch.until("\r\n  past thirty characters");
    let trees = watch.trees(Duration::ZERO);
    assert!(trees.len() >= 2, "{trees:?}");
}

/// On a terminal, `tree` prints the tree again as the times it shows change, with no record landing
/// (`spec/02-cli.md`, "Watching").
#[test]
fn watch_on_a_terminal_prints_the_tree_again_as_its_times_change() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let step = add(&repo, &["--under", &trail[..8], "--", "the step"]);
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("the step");
    watch.until("3s");
    assert!(watch.trees(Duration::ZERO).len() >= 2);
}

/// On an interrupt, `tree` leaves the alternate screen, prints the last tree whole and exits 0
/// (`spec/02-cli.md`, "Watching").
#[test]
fn watch_leaves_the_last_tree_on_an_interrupt() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("the trail");
    let pid = libc::pid_t::try_from(watch.child.id()).unwrap();
    assert_eq!(unsafe { libc::kill(pid, libc::SIGINT) }, 0);
    assert_eq!(watch.child.wait().unwrap().code(), Some(0));
    let shown = watch.shown(Duration::from_millis(200));
    let (_, after) = shown.split_once("\x1b[?1049l").expect("the main screen");
    assert!(after.contains("○ the trail\r\n"), "{shown:?}");
}

/// A watch spins a working step's mark and ends an active trail's second line with `active <time> ago`
/// (`spec/02-cli.md`, "Watching").
#[test]
fn watch_spins_a_working_step_and_says_when_the_trail_was_active() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let step = add(&repo, &["--under", &trail[..8], "--", "the step"]);
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("the step");
    let trees = watch.trees(Duration::from_millis(1000));
    let marks: std::collections::BTreeSet<char> = trees
        .iter()
        .flat_map(|t| t.chars())
        .filter(|c| SPIN.contains(*c))
        .collect();
    assert!(marks.len() >= 3, "{trees:?}");
    let last = trees.last().unwrap();
    assert!(!last.contains('◐'), "{last:?}");
    assert!(
        last.contains(" · active ") && last.contains(" ago"),
        "{last:?}"
    );
}

/// A watch cuts a tree taller than the screen to end at its last row with `… <n> more lines`
/// (`spec/02-cli.md`, "Watching").
#[test]
fn watch_cuts_a_tree_taller_than_the_screen() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    for n in 0..6 {
        add(&repo, &["--under", &trail[..8], "--", &format!("step {n}")]);
    }
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("the trail");
    watch.size(5, 80);
    watch.until("more lines");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    let lines: Vec<&str> = last.split("\r\n").collect();
    assert_eq!(lines.len(), 5, "{last:?}");
    assert_eq!(lines[3], "… 6 more lines", "{last:?}");
    assert!(lines[4].starts_with("↑↓ select"), "{last:?}");
}

/// A watch ends each tree with its keys, and `q` ends it as an interrupt does (`spec/02-cli.md`,
/// "Watching").
#[test]
fn watch_names_its_keys_and_quits_on_q() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("↑↓ select · enter open · f full · a all · q quit");
    watch.press("q");
    assert_eq!(watch.child.wait().unwrap().code(), Some(0));
    let shown = watch.shown(Duration::from_millis(200));
    let (_, after) = shown.split_once("\x1b[?1049l").expect("the main screen");
    assert!(
        after.contains("○ the trail") && !after.contains("q quit"),
        "{shown:?}"
    );
    assert!(!shown.contains("qthe trail"), "{shown:?}");
}

/// `↓` selects a trail in reverse video, and `enter` opens a done trail into a block and closes it
/// again (`spec/02-cli.md`, "Watching").
#[test]
fn watch_selects_a_trail_and_opens_it_with_enter() {
    let repo = Repo::new();
    let ended = add(&repo, &["--", "an ended trail"]);
    let part = add(&repo, &["--under", &ended[..8], "--", "its part"]);
    finish(&repo, &part[..8]);
    add(&repo, &["--", "an active trail"]);
    let mut watch = Watch::start(&repo, &[]);
    watch.until("DONE");
    watch.press("\x1b[B\x1b[B");
    std::thread::sleep(Duration::from_millis(300));
    let raw = String::from_utf8_lossy(&watch_raw(&mut watch)).into_owned();
    assert!(raw.contains("\x1b[7man ended trail"), "{raw:?}");
    watch.press("\r");
    watch.until("its part");
    watch.press("\r");
    let start = Instant::now();
    while watch
        .trees(Duration::from_millis(100))
        .last()
        .unwrap()
        .contains("its part")
    {
        assert!(start.elapsed() < Duration::from_secs(3), "still open");
    }
}

/// The raw bytes a watch printed so far.
fn watch_raw(watch: &mut Watch) -> Vec<u8> {
    watch.shown(Duration::ZERO);
    watch.seen.clone()
}

/// `f` turns the full tree on and off, and `a` every trail by day (`spec/02-cli.md`, "Watching").
#[test]
fn watch_turns_full_and_all_on_with_keys() {
    let repo = Repo::new();
    support::step_note(
        &repo,
        &support::account(),
        None,
        "an old trail",
        support::now() - 2 * support::DAY,
    );
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "the part"]);
    let o = cli(
        &repo,
        &["step", "done", &part[..8], "--as-planned", "--", "it works"],
    );
    assert!(o.status.success(), "{}", err(&o));
    add(&repo, &["--under", &trail[..8], "--", "next"]);
    let mut watch = Watch::start(&repo, &[]);
    watch.until("the part");
    watch.press("f");
    watch.until("done    it works");
    watch.press("f");
    watch.press("a");
    watch.until("an old trail");
}
