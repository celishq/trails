//! CIP-02, "Watching".

mod support;

use std::io::Read;
use std::os::fd::FromRawFd;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use support::{add, cli, err, finish, refusal, Repo};

/// The screen that clears before each tree.
const CLEAR: &str = "\x1b[H\x1b[2J\x1b[3J";

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

    /// The trees printed so far, one string each, after a wait of `wait` for more.
    fn trees(&mut self, wait: Duration) -> Vec<String> {
        self.shown(wait)
            .split(CLEAR)
            .skip(1)
            .map(str::to_string)
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

    /// Turns the terminal `columns` wide, and signals it.
    fn resize(&self, columns: u16) {
        let size = libc::winsize {
            ws_row: 24,
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
    watch.until(&format!("✓ {}", &trail[..8]));
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
    watch.until(&format!("✓ {}", &first[..8]));
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

/// On a terminal, `tree` clears the screen and its scrollback, and prints each tree from the top left, with
/// no empty line between two trees (`spec/02-cli.md`, "Watching").
#[test]
fn watch_on_a_terminal_clears_the_screen_before_each_tree() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("the trail");
    add(&repo, &["--under", &trail[..8], "--", "the step"]);
    watch.until("the step");
    let shown = watch.shown(Duration::ZERO);
    let trees = watch.trees(Duration::ZERO);
    assert!(trees.len() >= 2, "{shown:?}");
    assert!(shown.starts_with(CLEAR), "{shown:?}");
    assert!(!trees[0].contains("the step"), "{shown:?}");
    assert!(!shown.contains("\r\n\r\n"), "{shown:?}");
}

/// On a terminal, `tree` prints a tree whose times stand still again, cut to the new width, when the
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
    watch.until("…");
    let trees = watch.trees(Duration::ZERO);
    assert!(trees.len() >= 2, "{trees:?}");
    assert!(
        !trees[trees.len() - 1].contains("thirty characters"),
        "{trees:?}"
    );
}

/// On a terminal, `tree` prints the tree again as the times it shows change, with no record landing
/// (`spec/02-cli.md`, "Watching").
#[test]
fn watch_on_a_terminal_prints_the_tree_again_as_its_times_change() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("the trail");
    watch.until("3s");
    assert!(watch.trees(Duration::ZERO).len() >= 2);
}

/// On a terminal, `tree` prints the tree again within a second after a record lands in a linked folder that is
/// on (`spec/02-cli.md`, "Watching").
#[test]
fn watch_prints_the_tree_again_when_a_record_lands_in_a_linked_folder() {
    let repo = Repo::new();
    let o = cli(&repo, &["link"]);
    assert!(o.status.success(), "{}", err(&o));
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("the trail");
    let stranger = celis_nostr::Account::new(&[2u8; 32]).unwrap();
    let mut tags = support::step_tags((&repo.project_id(), "repo"));
    tags.push(vec!["under".into(), trail.clone()]);
    let note =
        celis_nostr::Note::write(&stranger, 1111u16, support::now(), tags, "Ben's step").unwrap();
    let path = repo.path().join(".celis/trails");
    let name = format!("{}-{}.json", note.created_at, note.id);
    std::fs::write(path.join(name), serde_json::to_string(&note).unwrap()).unwrap();
    let took = watch.until("Ben's step");
    assert!(took < Duration::from_millis(1500), "{took:?}");
}
