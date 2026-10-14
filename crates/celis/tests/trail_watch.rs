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
            .env("PATH", path(repo))
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

    /// The newest tree with its colour codes, after a wait of `wait` for more.
    fn coloured(&mut self, wait: Duration) -> String {
        self.shown(wait);
        let text = String::from_utf8_lossy(&self.seen).into_owned();
        text.rsplit(FRAME).next().unwrap_or("").to_string()
    }

    /// The lines of the newest tree, each with no codes, above the key line.
    fn rows(&mut self) -> Vec<String> {
        let last = self.trees(Duration::ZERO).pop().unwrap_or_default();
        let body = last.split("\x1b[").next().unwrap_or("").to_string();
        body.split("\r\n").map(str::to_string).collect()
    }

    /// Waits until the newest tree no longer holds `text`.
    fn without(&mut self, text: &str) {
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

/// The watch's `PATH`: the home's `bin` folder, then the test's own.
fn path(repo: &Repo) -> String {
    let own = std::env::var("PATH").unwrap_or_default();
    format!("{}:{own}", repo.home().join("bin").display())
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

    add(&repo, &["--under", &trail[..8], "--", "the step"]);
    let took = watch.until("the step");
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
    watch.until("the first trail");
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
    add(&repo, &["--", "the first trail"]);
    watch.until("the first trail");
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
    watch.until("\r\n    │ well past thirty");
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

/// On an interrupt, `tree` leaves the alternate screen, prints nothing and exits 0 (`spec/02-cli.md`,
/// "Watching").
#[test]
fn watch_leaves_nothing_on_an_interrupt() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("the trail");
    let pid = libc::pid_t::try_from(watch.child.id()).unwrap();
    assert_eq!(unsafe { libc::kill(pid, libc::SIGINT) }, 0);
    assert_eq!(watch.child.wait().unwrap().code(), Some(0));
    let shown = watch.shown(Duration::from_millis(200));
    let (_, after) = shown.split_once("\x1b[?1049l").expect("the main screen");
    assert_eq!(after, "", "{shown:?}");
}

/// A watch spins a working step's mark and puts `active <time> ago` under an active trail's title
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
        last.contains("│ active  ") && last.contains(" ago\r\n    │ repo"),
        "{last:?}"
    );
}

/// A watch prints its keys on the screen's last row, and scrolls a tree taller than the rows above
/// them, with `↑ <n> more` and `↓ <n> more` rows; `G`, `g` and `PgDn` scroll it (`spec/02-cli.md`,
/// "Watching").
#[test]
fn watch_scrolls_a_tree_taller_than_the_screen() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    for n in 0..20 {
        add(
            &repo,
            &["--under", &trail[..8], "--", &format!("step {n:02}")],
        );
    }
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("the trail");
    watch.size(10, 80);
    watch.until("↓ 15 more");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    assert!(last.contains("\x1b[10;1H↑↓ select"), "{last:?}");
    let rows: Vec<&str> = last
        .split("\x1b[10;1H")
        .next()
        .unwrap()
        .split("\r\n")
        .collect();
    assert_eq!(rows.len(), 9, "{last:?}");
    assert_eq!(rows[8], "  ↓ 15 more", "{last:?}");
    watch.press("G");
    watch.until("↑ 15 more");
    assert!(!watch.trees(Duration::ZERO).pop().unwrap().contains("  ↓ "));
    watch.press("g");
    watch.until("↓ 15 more");
    watch.press("\x1b[6~");
    watch.until("↑ 8 more");
}

/// A watch ends each tree with its keys, and `q` ends it as an interrupt does (`spec/02-cli.md`,
/// "Watching").
#[test]
fn watch_names_its_keys_and_quits_on_q() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until(
        "↑↓ select · ←→ close open · * open all under · PgUp PgDn scroll · f full · a all · q quit",
    );
    watch.press("q");
    assert_eq!(watch.child.wait().unwrap().code(), Some(0));
    let shown = watch.shown(Duration::from_millis(200));
    let (_, after) = shown.split_once("\x1b[?1049l").expect("the main screen");
    assert_eq!(after, "", "{shown:?}");
    assert!(!shown.contains("qthe trail"), "{shown:?}");
}

/// `↓` selects a trail with a `▌` in the cursor column in the text's colour and its title bold, and
/// `enter` opens a done trail into a block and closes it again (`spec/02-cli.md`, "Watching").
#[test]
fn watch_selects_a_trail_and_opens_it_with_enter() {
    let repo = Repo::new();
    let ended = add(&repo, &["--", "an ended trail"]);
    let part = add(&repo, &["--under", &ended[..8], "--", "its part"]);
    finish(&repo, &part[..8]);
    add(&repo, &["--", "an active trail"]);
    let mut watch = Watch::start(&repo, &[]);
    watch.until("an ended trail");
    watch.press("\x1b[B\x1b[B");
    std::thread::sleep(Duration::from_millis(300));
    let raw = String::from_utf8_lossy(&watch_raw(&mut watch)).into_owned();
    assert!(raw.contains("\r\n▌ \x1b[2m▸\x1b[0m"), "{raw:?}");
    assert!(raw.contains("\x1b[1man ended trail"), "{raw:?}");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    assert!(last.contains("\r\n▌ ▸ ✓ an ended trail"), "{last:?}");
    assert!(last.contains("\r\n  ▾ ○ an active trail"), "{last:?}");
    watch.press("\r");
    watch.until("▸ ✓ 1 closed step");
    watch.press("\r");
    watch.without("1 closed step");
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
    watch.until("1 closed step");
    watch.press("f");
    watch.until("done    it works");
    watch.until("the part");
    watch.press("f");
    watch.press("a");
    watch.until("an old trail");
}

/// `↓` selects the closed line of a block, `→` shows the steps in it, `enter` shows the lines `--full`
/// prints under the selected step, `c` copies its id's first 8 characters with `pbcopy`, `m` its list
/// item as Markdown, and each says so (`spec/02-cli.md`, "Watching").
#[test]
fn watch_opens_a_step_and_copies_its_id() {
    let repo = Repo::new();
    let bin = repo.home().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let copy = bin.join("pbcopy");
    std::fs::write(&copy, "#!/bin/sh\ncat > \"$CELIS_HOME/copied\"\n").unwrap();
    std::fs::set_permissions(&copy, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "the part"]);
    let o = cli(
        &repo,
        &["step", "done", &part[..8], "--as-planned", "--", "it works"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("1 closed step");
    assert!(!watch.rows().join("\n").contains("the part"));
    watch.press("\x1b[B\x1b[B");
    watch.until("\r\n▌   └▸ ✓ 1 closed step");
    watch.press("\x1b[C\x1b[B");
    watch.until("\r\n▌      └▸ ✓ the part");
    watch.until(&format!("c copy {}", &part[..8]));
    assert!(!watch.rows().join("\n").contains("it works"));
    watch.press("\r");
    watch.until("done    it works");
    let rows = watch.rows();
    let done = rows
        .iter()
        .find(|r| r.contains("done    it works"))
        .unwrap();
    assert!(done.starts_with("▌ "), "{rows:?}");
    watch.press("c");
    watch.until(&format!("copied {}", &part[..8]));
    let copied = std::fs::read_to_string(repo.home().join("copied")).unwrap();
    assert_eq!(copied, &part[..8]);
    watch.press("m");
    watch.until("copied markdown");
    let copied = std::fs::read_to_string(repo.home().join("copied")).unwrap();
    assert!(
        copied.starts_with("- ✓ the part\n") && copied.contains("done: it works"),
        "{copied}"
    );
}

/// A block folds its rules to one line, and `→` opens it to a line a rule, each wrapped whole; the
/// watch prints no ids (`spec/02-cli.md`, "Watching").
#[test]
fn watch_opens_the_rules_line() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    add(&repo, &["--under", &trail[..8], "--", "the part"]);
    let rule = "When you price a senior: use cents, because floats lost a cent on the receipt.";
    let scope = format!("trail:{}", &trail[..8]);
    let o = cli(&repo, &["decide", "--scope", &scope, "--", rule]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.size(24, 60);
    watch.until("├▸ § 1 rule");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    assert!(
        !last.contains("When you price") && !last.contains(&trail[..8]),
        "{last:?}"
    );
    watch.press("\x1b[B\x1b[B");
    watch.until("\r\n▌   ├▸ § 1 rule");
    watch.press("\x1b[C");
    watch.until("├▾ § 1 rule");
    watch.until("│  └─ § When you price");
    watch.until("the receipt");
}

/// `*` opens the selected step and every step under it, and closes them all again; `←` on a closed
/// step selects the step it is under, `→` opens it and `←` closes it (`spec/02-cli.md`, "Watching").
#[test]
fn watch_opens_and_closes_with_arrows_and_star() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let parent = add(
        &repo,
        &["--under", &trail[..8], "--", "parent\nparent text"],
    );
    add(&repo, &["--under", &parent[..8], "--", "child\nchild text"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.until("child");
    watch.press("\x1b[B\x1b[B");
    watch.until("▌   └▸ ○ parent");
    watch.press("*");
    watch.until("parent text");
    watch.until("child text");
    watch.press("*");
    watch.without("parent text");
    watch.without("child text");
    watch.press("\x1b[B");
    watch.until("▌      └▸ ○ child");
    watch.press("\x1b[D");
    watch.until("▌   └▸ ○ parent");
    watch.press("\x1b[C");
    watch.until("parent text");
    assert!(!watch
        .trees(Duration::ZERO)
        .pop()
        .unwrap()
        .contains("child text"));
    watch.press("\x1b[D");
    watch.without("parent text");
}

/// The watch draws a block with its lines under its title, its tree lines, its rules folded to one
/// line, its details in columns and no ids, in colour: § magenta, places blue, the project bold
/// (`spec/02-cli.md`, "Watching").
#[test]
fn watch_draws_a_block_with_tree_lines_and_colour_anchors() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let done = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--at",
            "shelf/prices.py:40",
            "--",
            "done part",
        ],
    );
    let deeper = add(&repo, &["--under", &done[..8], "--", "deeper part"]);
    let scope = format!("trail:{}", &trail[..8]);
    let o = cli(&repo, &["decide", "--scope", &scope, "--", "a rule"]);
    assert!(o.status.success(), "{}", err(&o));
    let o = cli(&repo, &["step", "start", &deeper[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    std::thread::sleep(Duration::from_millis(1100));
    add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--at",
            "shelf/a.py",
            "--",
            "the fix",
        ],
    );
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.size(24, 80);
    watch.until("the fix");
    let rows = watch.rows();
    assert!(rows[0].starts_with("  ▾ "), "{rows:?}");
    assert!(rows[1].starts_with("    │ took    "), "{rows:?}");
    assert!(
        rows[2].starts_with("    │ active  ") && rows[2].ends_with(" ago"),
        "{rows:?}"
    );
    assert_eq!(rows[3], "    │ repo", "{rows:?}");
    assert_eq!(rows[4], "    ├▸ § 1 rule", "{rows:?}");
    assert!(rows[5].starts_with("    ├─ ○ done part  "), "{rows:?}");
    assert!(rows[6].starts_with("    │  └─ "), "{rows:?}");
    assert!(
        !rows[5].contains("prices.py") && rows[5].ends_with('s'),
        "{rows:?}"
    );
    assert!(rows[7] == "    └─ ○ the fix", "{rows:?}");
    assert!(!rows.join("\n").contains(&trail[..8]), "{rows:?}");
    let raw = watch.coloured(Duration::ZERO);
    for anchor in ["\x1b[35m§", "\x1b[1mrepo"] {
        assert!(raw.contains(anchor), "{anchor:?} in {raw:?}");
    }
}

/// A step whose newest record is less than 2 seconds old prints its title bold, then in normal weight;
/// the closed line's mark is green (`spec/02-cli.md`, "Watching").
#[test]
fn watch_prints_a_fresh_step_bold() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let done = add(&repo, &["--under", &trail[..8], "--", "a done part"]);
    finish(&repo, &done[..8]);
    add(&repo, &["--under", &trail[..8], "--", "the part"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.size(24, 80);
    watch.until("the part");
    assert!(watch.coloured(Duration::ZERO).contains("\x1b[1mthe part"));
    std::thread::sleep(Duration::from_millis(2500));
    let raw = watch.coloured(Duration::from_millis(300));
    assert!(!raw.contains("\x1b[1mthe part"), "{raw:?}");
    assert!(raw.contains("\x1b[32m✓\x1b[0m 1 closed step"), "{raw:?}");
}

/// The closed line holds the done and dropped steps, marked `✓✗`; open, a done step with steps under it
/// prints them only when it is open, its line holding their count and no time while closed
/// (`spec/02-cli.md`, "Watching").
#[test]
fn watch_folds_closed_steps_and_done_branches() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let parent = add(&repo, &["--under", &trail[..8], "--", "parent"]);
    let child = add(&repo, &["--under", &parent[..8], "--", "child"]);
    finish(&repo, &child[..8]);
    finish(&repo, &parent[..8]);
    std::thread::sleep(Duration::from_millis(1100));
    let gone = add(&repo, &["--under", &trail[..8], "--", "gone"]);
    let o = cli(&repo, &["step", "drop", &gone[..8], "--", "not needed"]);
    assert!(o.status.success(), "{}", err(&o));
    std::thread::sleep(Duration::from_millis(1100));
    add(&repo, &["--under", &trail[..8], "--", "next"]);
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.size(24, 80);
    watch.until("next");
    let rows = watch.rows();
    assert!(
        rows.iter().any(|r| r.ends_with("├▸ ✓✗ 2 closed steps")),
        "{rows:?}"
    );
    assert!(!rows.join("\n").contains("parent"), "{rows:?}");
    watch.press("\x1b[B\x1b[B\x1b[C");
    watch.until("parent");
    let rows = watch.rows();
    let line = rows.iter().find(|r| r.contains("parent")).unwrap();
    assert!(line.ends_with("1 step "), "{rows:?}");
    assert!(!rows.join("\n").contains("child") && !rows.join("\n").contains("not needed"));
    watch.press("\x1b[B\x1b[C");
    watch.until("✓ child");
}

/// With no id, the watch starts with today's line and its counts, groups the trails by project, and
/// ends with `Show more`, which `enter` turns into every trail by day and `Show less`
/// (`spec/02-cli.md`, "Watching").
#[test]
fn watch_groups_today_by_project_and_shows_more() {
    let repo = Repo::new();
    support::step_note(
        &repo,
        &support::account(),
        None,
        "an old trail",
        support::now() - 2 * support::DAY,
    );
    add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[]);
    watch.size(24, 80);
    watch.until("▸ Show more");
    let rows = watch.rows();
    assert!(rows[0].ends_with(" · 1 active · 0 done"), "{rows:?}");
    assert!(rows[1].trim().is_empty(), "{rows:?}");
    assert!(
        rows[2].starts_with("  ▾ ○ the trail") && rows[2].ends_with(" repo"),
        "{rows:?}"
    );
    assert!(!rows.join("\n").contains("an old trail"), "{rows:?}");
    watch.press("\x1b[B\x1b[B");
    watch.until("▌ ▸ Show more");
    watch.press("\r");
    watch.until("▾ Show less");
    watch.until("an old trail");
}

/// Under 80 columns, while a title would be cut below 20 characters, the watch leaves out a trail
/// line's first column (`spec/02-cli.md`, "Watching").
#[test]
fn a_narrow_watch_leaves_out_columns() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "a trail with a long title of many words"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "its part"]);
    finish(&repo, &part[..8]);
    let mut watch = Watch::start(&repo, &[]);
    watch.size(24, 100);
    watch.until("a trail with");
    let line = |watch: &mut Watch| {
        let rows = watch.rows();
        rows.iter()
            .find(|r| r.contains("✓ a trail"))
            .unwrap()
            .clone()
    };
    let wide = line(&mut watch);
    assert!(wide.contains("1 step") && wide.ends_with('s'), "{wide}");
    watch.size(24, 30);
    watch.without("1 step");
    assert!(line(&mut watch).contains('…'), "{}", line(&mut watch));
}

/// A trail's line holds its steps and its rules in their own columns, each number aligned right
/// (`spec/02-cli.md`, "Watching").
#[test]
fn watch_aligns_a_trail_lines_steps_and_rules() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 4_000;
    let one = support::step_note(&repo, &by, None, "first", at);
    let part = support::step_note(&repo, &by, Some(&one), "part", at);
    support::done_note(&repo, &by, &part, at + 5);
    let many = support::step_note(&repo, &by, None, "second", at + 10);
    for n in 0..12 {
        let part = support::step_note(&repo, &by, Some(&many), &format!("part {n}"), at + 10);
        support::done_note(&repo, &by, &part, at + 20);
    }
    let project = repo.project_id();
    for n in 0..10 {
        let scope = format!("trail:{}", one.id);
        support::decision(
            &repo,
            (&project, "repo"),
            &by,
            (Some(&scope), None),
            &format!("rule {n}"),
            at,
        );
    }
    let mut watch = Watch::start(&repo, &[]);
    watch.size(24, 100);
    watch.until("second");
    let rows = watch.rows();
    let line = |what: &str| rows.iter().find(|l| l.contains(what)).unwrap().clone();
    let (one, many) = (line("first"), line("second"));
    assert!(one.contains("   1 step   10 rules"), "{rows:?}");
    assert!(many.contains("  12 steps          "), "{rows:?}");
    assert_eq!(one.find(" step"), many.find(" step"), "{rows:?}");
}

/// With `--full`, the watch starts each line under a step at its title's column, after its label, and
/// wraps it at the width (`spec/02-cli.md`, "Watching").
#[test]
fn watch_full_aligns_and_wraps_the_lines_under_a_step() {
    let repo = Repo::new();
    let long =
        "one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen";
    let trail = add(&repo, &["--", "the trail"]);
    let first = add(&repo, &["--under", &trail[..8], "--", "first"]);
    let o = cli(
        &repo,
        &["step", "done", &first[..8], "--as-planned", "--", long],
    );
    assert!(o.status.success(), "{}", err(&o));
    std::thread::sleep(Duration::from_millis(1100));
    add(&repo, &["--under", &trail[..8], "--", "second"]);
    let mut watch = Watch::start(&repo, &[&trail[..8], "--full"]);
    watch.size(24, 60);
    watch.until("fifteen");
    let rows = watch.rows();
    assert!(rows.iter().all(|r| r.chars().count() <= 60), "{rows:?}");
    let at = rows
        .iter()
        .position(|r| r.contains(" done    one "))
        .unwrap_or_else(|| panic!("{rows:?}"));
    let column = rows[at].find("done    ").unwrap() + "done    ".len();
    let mut text = rows[at][column..].trim_end().to_string();
    for row in &rows[at + 1..] {
        let lead = row.get(..column).unwrap_or("x");
        match row.get(column..) {
            Some(more) if lead.chars().all(|c| c == ' ' || c == '│') && !more.is_empty() => {
                text = format!("{text} {}", more.trim_end())
            }
            _ => break,
        }
    }
    assert_eq!(text, long, "{rows:?}");
}

/// When `a` takes the selected trail of a past day out of the tree, the watch selects the first line,
/// and the arrows move on from it (`spec/02-cli.md`, "Watching").
#[test]
fn watch_keeps_a_selection_when_its_line_leaves() {
    let repo = Repo::new();
    support::step_note(
        &repo,
        &support::account(),
        None,
        "an old trail",
        support::now() - 2 * support::DAY,
    );
    add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[]);
    watch.size(24, 80);
    watch.until("▸ Show more");
    watch.press("a");
    watch.until("an old trail");
    watch.press("\x1b[B\x1b[B");
    watch.until("▌ ▸ ✓ an old trail");
    watch.press("a");
    watch.until("▌ ▾ ○ the trail");
    watch.press("\x1b[B");
    watch.until("▌ ▸ Show more");
}

/// The key line is cut to the width, so it never wraps and scrolls the screen (`spec/02-cli.md`,
/// "Watching").
#[test]
fn watch_cuts_the_key_line_to_the_width() {
    let repo = Repo::new();
    add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &[]);
    watch.size(24, 40);
    watch.until("\x1b[24;1H");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    let keys = last.split("\x1b[24;1H").nth(1).unwrap();
    assert!(keys.chars().count() <= 40, "{keys:?}");
    assert!(keys.ends_with('…'), "{keys:?}");
}

/// A changed done step prints `~` after its mark, and no step prints its place (`spec/02-cli.md`,
/// "Watching").
#[test]
fn watch_marks_a_change_and_prints_no_place() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--at",
            "shelf/a.py",
            "--",
            "the part",
        ],
    );
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &part[..8],
            "--changed",
            "one file",
            "--",
            "it works",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let next = add(&repo, &["--under", &trail[..8], "--", "next"]);
    let o = cli(&repo, &["step", "start", &next[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &[&trail[..8]]);
    watch.size(24, 80);
    watch.until("1 closed step");
    watch.press("\x1b[B\x1b[B\x1b[C");
    watch.until("✓~ the part");
    let rows = watch.rows();
    let row = rows.iter().find(|r| r.contains("the part")).unwrap();
    assert!(row.ends_with("✓~ the part"), "{rows:?}");
}

/// No line of the watch reaches the last column, so clearing to the end of a line never erases its
/// last character (`spec/02-cli.md`, "Watching").
#[test]
fn watch_leaves_the_last_column_empty() {
    let repo = Repo::new();
    let trail = add(
        &repo,
        &["--", "a trail whose title runs well past forty characters"],
    );
    let step = add(&repo, &["--under", &trail[..8], "--", "a working step"]);
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &[]);
    watch.size(24, 40);
    watch.until("a working step");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    for row in last.replace("\x1b[24;1H", "\r\n").split("\r\n") {
        assert!(row.chars().count() < 40, "{row:?}");
    }
    let rows = watch.rows();
    let line = rows.iter().find(|r| r.contains("a working step")).unwrap();
    assert!(line.ends_with('s'), "{rows:?}");
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
