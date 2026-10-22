//! The view `tree` shows on a terminal.

mod support;

use std::time::Duration;
use support::watch::Watch;
use support::{add, cli, err, finish, refusal, Repo};

/// The spinner's frames.
const SPIN: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

fn records(repo: &Repo) -> usize {
    std::fs::read_dir(repo.store().join("shared"))
        .unwrap()
        .count()
}

/// On a terminal, `tree <id>` prints the tree, then prints it again within a second after a record lands,
/// and writes no record.
#[test]
fn watch_prints_the_tree_again_when_a_record_lands() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.until("the trail");
    let quiet = records(&repo);
    watch.trees(Duration::from_millis(1200));
    assert_eq!(records(&repo), quiet);

    add(&repo, &["--under", &trail[..8], "--", "the step"]);
    let took = watch.until("the step");
    assert!(took < Duration::from_millis(1500), "{took:?}");
}

/// On a terminal, `tree <id>` of a trail whose times stand still prints it again only when a record
/// changes the tree.
#[test]
fn watch_prints_the_tree_again_only_when_it_changes() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    finish(&repo, &trail[..8]);
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
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
/// prints: the trails of every project.
#[test]
fn watch_with_no_id_prints_what_tree_prints() {
    let repo = Repo::new();
    let mut watch = Watch::start(&repo, &["tree"]);
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
/// shows the first trail once another command writes it.
#[test]
fn watch_on_an_empty_home_writes_nothing_and_waits_for_the_store() {
    let repo = Repo::new();
    std::fs::remove_file(repo.home().join("phrase")).unwrap();
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.until("No active trails");
    assert!(!repo.home().join("phrase").exists());
    assert!(!repo.store().exists());
    add(&repo, &["--", "the first trail"]);
    watch.until("the first trail");
}

/// On a terminal, `tree` refuses an id no step has (`spec/02-cli.md`, "Output and errors").
#[test]
fn watch_refuses_an_unknown_id() {
    let repo = Repo::new();
    add(&repo, &["--", "a trail"]);
    let mut watch = Watch::start(&repo, &["tree", "deadbeef"]);
    assert_eq!(watch.child.wait().unwrap().code(), Some(1));
    assert!(
        watch
            .shown(Duration::from_millis(200))
            .contains("no step has the id deadbeef"),
        "{:?}",
        watch.shown(Duration::ZERO)
    );
}

/// `tree` takes no `--watch`: it is a usage error (`spec/02-cli.md`, "Commands").
#[test]
fn tree_refuses_watch() {
    let repo = Repo::new();
    refusal(&cli(&repo, &["tree", "--watch", "--json"]), 2, "usage");
}

/// On a terminal, `tree` draws on the alternate screen, each tree from the top left over the last one, with
/// no clear.
#[test]
fn watch_draws_each_tree_over_the_last_on_the_alternate_screen() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.until("the trail");
    add(&repo, &["--under", &trail[..8], "--", "the step"]);
    watch.until("the step");
    let shown = watch.shown(Duration::ZERO);
    let trees = watch.trees(Duration::ZERO);
    assert!(trees.len() >= 2, "{shown:?}");
    assert!(
        shown
            .trim_start_matches("\x1b]11;?\x07")
            .starts_with("\x1b[?1049h"),
        "{shown:?}"
    );
    assert!(!trees[0].contains("the step"), "{shown:?}");
    assert!(!shown.contains("\x1b[2J"), "{shown:?}");
}

/// On a terminal, `tree` prints a tree whose times stand still again, wrapped to the new width, when the
/// terminal's width changes.
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
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.until("thirty characters");
    watch.resize(30);
    watch.until("\r\n    │ well past thirty");
    let trees = watch.trees(Duration::ZERO);
    assert!(trees.len() >= 2, "{trees:?}");
}

/// On a terminal, `tree` prints the tree again as the times it shows change, with no record
/// landing.
#[test]
fn watch_on_a_terminal_prints_the_tree_again_as_its_times_change() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let step = add(&repo, &["--under", &trail[..8], "--", "the step"]);
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.until("the step");
    watch.until("3s");
    assert!(watch.trees(Duration::ZERO).len() >= 2);
}

/// On an interrupt, `tree` leaves the alternate screen, prints nothing and exits 0.
#[test]
fn watch_leaves_nothing_on_an_interrupt() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.until("the trail");
    let pid = libc::pid_t::try_from(watch.child.id()).unwrap();
    assert_eq!(unsafe { libc::kill(pid, libc::SIGINT) }, 0);
    assert_eq!(watch.child.wait().unwrap().code(), Some(0));
    let shown = watch.shown(Duration::from_millis(200));
    let (_, after) = shown.split_once("\x1b[?1049l").expect("the main screen");
    assert_eq!(after, "", "{shown:?}");
}

/// A watch spins a working step's mark and puts `active <time> ago` under an active trail's title.
#[test]
fn watch_spins_a_working_step_and_says_when_the_trail_was_active() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let step = add(&repo, &["--under", &trail[..8], "--", "the step"]);
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
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
        last.contains("│ active   ") && last.contains(" ago\r\n    │ repo"),
        "{last:?}"
    );
}

/// A watch prints its keys on the screen's last row, and scrolls a tree taller than the rows above
/// them, with `↑ <n> more` and `↓ <n> more` rows; `G`, `g` and `PgDn` scroll it.
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
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
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

/// A watch ends each tree with its keys, and `q` ends it as an interrupt does.
#[test]
fn watch_names_its_keys_and_quits_on_q() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
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
/// `enter` opens a done trail into a block and closes it again.
#[test]
fn watch_selects_a_trail_and_opens_it_with_enter() {
    let repo = Repo::new();
    let ended = add(&repo, &["--", "an ended trail"]);
    let part = add(&repo, &["--under", &ended[..8], "--", "its part"]);
    finish(&repo, &part[..8]);
    add(&repo, &["--", "an active trail"]);
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.until("an ended trail");
    watch.press("\x1b[B\x1b[B");
    std::thread::sleep(Duration::from_millis(300));
    let raw = watch.coloured(Duration::ZERO);
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

/// A run of `↓` typed at once moves the selection by each key, and draws one tree for the run, not one
/// for each key.
#[test]
fn watch_draws_a_run_of_keys_as_one_tree() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    for n in 1..=12 {
        add(
            &repo,
            &["--under", &trail[..8], "--", &format!("step {n:02}")],
        );
    }
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.until("step 01");
    let before = watch.trees(Duration::from_millis(300)).len();
    let tenth = watch.rows()[11].clone();
    watch.press(&"\x1b[B".repeat(10));
    watch.until(&format!("▌{}", &tenth[1..]));
    let drawn = watch.trees(Duration::from_millis(300)).len() - before;
    assert!(drawn < 4, "{drawn} trees for 10 keys");
}

/// A spinner tick rewrites only the rows that changed: no row of a step that stands still.
#[test]
fn watch_rewrites_only_the_rows_a_tick_changes() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 10;
    let trail = support::step_note(&repo, &by, None, "the trail", at);
    for n in 1..=8 {
        support::step_note(&repo, &by, Some(&trail), &format!("step {n}"), at);
    }
    let step = support::step_note(&repo, &by, Some(&trail), "the step", at);
    support::put(
        &repo,
        &celis_trails::Mark::Start.record(&by, &step.id, at).unwrap(),
    );
    let mut watch = Watch::start(&repo, &["tree", &trail.id[..8]]);
    watch.until("the step");
    let raw = String::from_utf8_lossy(&watch_raw(&mut watch)).into_owned();
    let first = raw.matches("\x1b[?2026h").count();
    std::thread::sleep(Duration::from_millis(600));
    let raw = String::from_utf8_lossy(&watch_raw(&mut watch)).into_owned();
    let frames: Vec<&str> = raw.split("\x1b[?2026h").skip(1).collect();
    let ticks = &frames[first..];
    assert!(ticks.len() >= 3, "{frames:?}");
    for tick in ticks {
        assert!(!tick.contains("step "), "{tick:?}");
    }
}

/// A row many levels deep wraps its title within the width.
#[test]
fn watch_wraps_a_deep_row_within_the_width() {
    let repo = Repo::new();
    let mut under = add(&repo, &["--", "the trail"]);
    for n in 1..=8 {
        under = add(
            &repo,
            &["--under", &under[..8], "--", &format!("level {n}")],
        );
    }
    let words = "alpha bravo charlie delta echo foxtrot golf hotel";
    add(
        &repo,
        &["--under", &under[..8], "--", &format!("deep {words}")],
    );
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.size(40, 40);
    watch.until("deep");
    let rows = watch.rows();
    assert!(rows.iter().all(|r| r.chars().count() <= 40), "{rows:?}");
    let text: String = rows.concat().split_whitespace().collect();
    let words: String = format!("deep {words}").split_whitespace().collect();
    assert!(text.contains(&words), "{rows:?}");
}

/// The raw bytes a watch printed so far.
fn watch_raw(watch: &mut Watch) -> Vec<u8> {
    watch.shown(Duration::ZERO);
    watch.seen.clone()
}

/// `f` turns the full tree on and off, and `a` every trail by day.
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
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.until("1 closed step");
    watch.press("f");
    watch.until("done     it works");
    watch.until("the part");
    watch.press("f");
    watch.press("a");
    watch.until("an old trail");
}

/// `↓` selects the closed line of a block, `→` shows the steps in it, `enter` shows the lines `--full`
/// prints under the selected step, `c` copies its id's first 8 characters with `pbcopy`, `m` its table
/// as Markdown, and each says so.
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
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.until("1 closed step");
    assert!(!watch.rows().join("\n").contains("the part"));
    watch.press("\x1b[B\x1b[B");
    watch.until("\r\n▌   └▸ ✓ 1 closed step");
    watch.press("\x1b[C\x1b[B");
    watch.until("\r\n▌      └▸ ✓ the part");
    watch.until(&format!("c copy {}", &part[..8]));
    assert!(!watch.rows().join("\n").contains("it works"));
    watch.press("\r");
    watch.until("done     it works");
    let rows = watch.rows();
    let done = rows
        .iter()
        .find(|r| r.contains("done     it works"))
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
        copied.starts_with("| | Step | Took | Proof |\n") && copied.contains(" | it works |"),
        "{copied}"
    );
}

/// A block folds its rules to one line, and `→` opens it to a line a rule, each wrapped whole; the
/// watch prints no ids.
#[test]
fn watch_opens_the_rules_line() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    add(&repo, &["--under", &trail[..8], "--", "the part"]);
    let rule = "When you price a senior: use cents, because floats lost a cent on the receipt.";
    let scope = format!("trail:{}", &trail[..8]);
    let o = cli(&repo, &["decide", "--scope", &scope, "--", rule]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.size(24, 60);
    watch.until("└▸ § 1 rule");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    assert!(
        !last.contains("When you price") && !last.contains(&trail[..8]),
        "{last:?}"
    );
    watch.press("\x1b[B\x1b[B\x1b[B");
    watch.until("\r\n▌   └▸ § 1 rule");
    watch.press("\x1b[C");
    watch.until("└▾ § 1 rule");
    watch.until("   └─ § When you price");
    watch.until("the receipt");
}

/// `*` opens the selected step and every step under it, and closes them all again; `←` on a closed
/// step selects the step it is under, `→` opens it and `←` closes it.
#[test]
fn watch_opens_and_closes_with_arrows_and_star() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let parent = add(
        &repo,
        &["--under", &trail[..8], "--", "parent\nparent text"],
    );
    add(&repo, &["--under", &parent[..8], "--", "child\nchild text"]);
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
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
/// line, each step's details on a line under its title and no ids, in colour: § magenta, places blue, the project bold.
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
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.size(24, 80);
    watch.until("the fix");
    let rows = watch.rows();
    assert!(rows[0].starts_with("  ▾ "), "{rows:?}");
    assert!(rows[1].starts_with("    │ took     "), "{rows:?}");
    assert!(
        rows[2].starts_with("    │ active   ") && rows[2].ends_with(" ago"),
        "{rows:?}"
    );
    assert_eq!(rows[3], "    │ repo", "{rows:?}");
    assert!(rows[4] == "    ├─ ○ the fix", "{rows:?}");
    assert_eq!(rows[5], "    ├─ ○ done part", "{rows:?}");
    assert!(
        rows[6].starts_with("    │  │ ") && rows[6].ends_with('s'),
        "{rows:?}"
    );
    assert!(rows[7].starts_with("    │  └─ "), "{rows:?}");
    assert!(!rows.join("\n").contains("prices.py"), "{rows:?}");
    assert_eq!(rows[9], "    └▸ § 1 rule", "{rows:?}");
    assert!(!rows.join("\n").contains(&trail[..8]), "{rows:?}");
    let raw = watch.coloured(Duration::ZERO);
    for anchor in ["\x1b[35m§", "\x1b[1mrepo"] {
        assert!(raw.contains(anchor), "{anchor:?} in {raw:?}");
    }
}

/// A step whose newest record is less than 2 seconds old prints its title bold, then in normal weight;
/// the closed line's mark is green.
#[test]
fn watch_prints_a_fresh_step_bold() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let done = add(&repo, &["--under", &trail[..8], "--", "a done part"]);
    finish(&repo, &done[..8]);
    add(&repo, &["--under", &trail[..8], "--", "the part"]);
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.size(24, 80);
    watch.until("the part");
    assert!(watch.coloured(Duration::ZERO).contains("\x1b[1mthe part"));
    std::thread::sleep(Duration::from_millis(2500));
    let raw = watch.coloured(Duration::from_millis(300));
    assert!(!raw.contains("\x1b[1mthe part"), "{raw:?}");
    assert!(raw.contains("\x1b[32m✓\x1b[0m 1 closed step"), "{raw:?}");
}

/// A block prints its open steps newest first, then its rules line, then its closed line, and a step
/// folds its own closed steps the same way.
#[test]
fn watch_prints_open_steps_then_rules_then_closed_steps() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 60;
    let trail = support::step_note(&repo, &by, None, "the trail", at);
    let scope = format!("trail:{}", trail.id);
    let project = (repo.project_id(), "repo");
    let rule = (Some(scope.as_str()), None);
    support::stored_rule(&repo, (&project.0, project.1), &by, rule, "a rule", at);
    let finished = support::step_note(&repo, &by, Some(&trail), "finished", at + 1);
    support::mark_note(&repo, &by, "done", &finished.id, "proof", at + 2);
    let parent = support::step_note(&repo, &by, Some(&trail), "parent", at + 3);
    let old = support::step_note(&repo, &by, Some(&parent), "old child", at + 4);
    support::mark_note(&repo, &by, "done", &old.id, "proof", at + 5);
    support::step_note(&repo, &by, Some(&parent), "new child", at + 6);
    support::step_note(&repo, &by, Some(&trail), "newest", at + 7);
    let mut watch = Watch::start(&repo, &["tree", &trail.id[..8]]);
    watch.size(24, 80);
    watch.until("newest");
    let rows = watch.rows();
    let tree: Vec<&str> = rows
        .iter()
        .map(|r| r.trim_end())
        .skip_while(|r| !r.contains("newest"))
        .collect();
    let starts = [
        "    ├─ ○ newest",
        "    ├─ ○ parent",
        "    │  │ ",
        "    │  ├─ ○ new child",
        "    │  └▸ ✓ 1 closed step",
        "    ├▸ § 1 rule",
        "    └▸ ✓ 1 closed step",
    ];
    assert_eq!(tree.len(), starts.len(), "{rows:?}");
    for (row, start) in tree.iter().zip(starts) {
        assert!(row.starts_with(start), "{start:?} in {rows:?}");
    }
}

/// The closed line holds a trail's expired steps too, and marks them `◌` after `✓` and `✗`.
#[test]
fn watch_holds_expired_steps_in_the_closed_line() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 2 * 3600;
    let trail = support::step_note(&repo, &by, None, "the trail", at);
    let done = support::step_note(&repo, &by, Some(&trail), "done part", at + 1);
    support::mark_note(&repo, &by, "done", &done.id, "proof", at + 2);
    support::step_note(&repo, &by, Some(&trail), "left part", at + 3);
    let mut watch = Watch::start(&repo, &["tree", &trail.id[..8]]);
    watch.size(24, 80);
    watch.until("2 closed steps");
    let rows = watch.rows();
    assert!(
        rows.iter().any(|r| r.ends_with("└▸ ✓◌ 2 closed steps")),
        "{rows:?}"
    );
    assert!(!rows.join("\n").contains("left part"), "{rows:?}");
}

/// The closed line holds the done and dropped steps, marked `✓✗`; open, a done step with steps under it
/// prints its own closed line only when it is open, its line holding their count and no time while
/// closed.
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
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.size(24, 80);
    watch.until("next");
    let rows = watch.rows();
    assert!(
        rows.iter().any(|r| r.ends_with("└▸ ✓✗ 2 closed steps")),
        "{rows:?}"
    );
    assert!(!rows.join("\n").contains("parent"), "{rows:?}");
    watch.press("\x1b[B\x1b[B\x1b[B\x1b[C");
    watch.until("parent");
    let rows = watch.rows();
    let at = rows.iter().position(|r| r.contains("parent")).unwrap();
    assert_eq!(rows[at + 1].trim(), "1 step", "{rows:?}");
    assert!(!rows.join("\n").contains("child") && !rows.join("\n").contains("not needed"));
    watch.press("\x1b[B\x1b[B\x1b[C");
    watch.until("✓  1 closed step");
    watch.press("\x1b[B\x1b[C");
    watch.until("✓  child");
}

/// With no id, the watch starts with today's line and its counts, groups the trails by project, and
/// ends with `Show more`, which `enter` turns into every trail by day and `Show less`.
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
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.size(24, 80);
    watch.until("▸ Show more");
    let rows = watch.rows();
    assert!(rows[0].ends_with(" · 1 active · 0 done"), "{rows:?}");
    assert!(rows[1].trim().is_empty(), "{rows:?}");
    assert_eq!(rows[2], "  ▾ ○ the trail", "{rows:?}");
    assert!(rows.contains(&"    │ repo".to_string()), "{rows:?}");
    assert!(!rows.join("\n").contains("an old trail"), "{rows:?}");
    watch.press("\x1b[B\x1b[B");
    watch.until("▌ ▸ Show more");
    watch.press("\r");
    watch.until("▾ Show less");
    watch.until("an old trail");
}

/// The watch wraps a long step title and a long trail line's title whole, cutting neither.
#[test]
fn watch_wraps_long_titles_whole() {
    let repo = Repo::new();
    let words = "one two three four five six seven eight nine ten eleven twelve";
    let done = add(&repo, &["--", &format!("done trail {words}")]);
    let part = add(&repo, &["--under", &done[..8], "--", "its part"]);
    finish(&repo, &part[..8]);
    let trail = add(&repo, &["--", "the trail"]);
    add(
        &repo,
        &["--under", &trail[..8], "--", &format!("open step {words}")],
    );
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.size(24, 40);
    watch.until("open step");
    let rows = watch.rows();
    let text = rows.join(" ");
    assert!(!text.contains('…'), "{rows:?}");
    assert!(rows.iter().all(|r| r.chars().count() <= 40), "{rows:?}");
    for title in ["done trail", "open step"] {
        let at = rows.iter().position(|r| r.contains(title)).unwrap();
        let end = rows
            .iter()
            .skip(at)
            .position(|r| r.contains("twelve"))
            .unwrap();
        let block = rows[at..=at + end].join(" ").replace("1 step", "");
        let block: Vec<&str> = block.split_whitespace().collect();
        assert!(
            block.join(" ").contains(&format!("{title} {words}")),
            "{rows:?}"
        );
    }
}

/// A trail's line prints its steps and worked time on a dim line under its title, and no count of its
/// rules.
#[test]
fn watch_prints_a_trail_lines_details_under_its_title() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 4_000;
    let one = support::step_note(&repo, &by, None, "first", at);
    let part = support::step_note(&repo, &by, Some(&one), "part", at);
    support::mark_note(&repo, &by, "done", &part.id, "proof", at + 5);
    let many = support::step_note(&repo, &by, None, "second", at + 10);
    for n in 0..12 {
        let part = support::step_note(&repo, &by, Some(&many), &format!("part {n}"), at + 10);
        support::mark_note(&repo, &by, "done", &part.id, "proof", at + 20);
    }
    let project = repo.project_id();
    for n in 0..10 {
        let scope = format!("trail:{}", one.id);
        support::stored_rule(
            &repo,
            (&project, "repo"),
            &by,
            (Some(&scope), None),
            &format!("rule {n}"),
            at,
        );
    }
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.size(24, 100);
    watch.until("second");
    let rows = watch.rows();
    let under = |what: &str| {
        let at = rows.iter().position(|l| l.contains(what)).unwrap();
        (
            rows[at].trim_end().to_string(),
            rows[at + 1].trim().to_string(),
        )
    };
    assert_eq!(
        under("first"),
        ("  ▸ ✓ first".into(), "1 step · 5s".into()),
        "{rows:?}"
    );
    assert_eq!(
        under("second"),
        ("  ▸ ✓ second".into(), "12 steps · 2m 0s".into()),
        "{rows:?}"
    );
    assert!(!rows.join("\n").contains("rule"), "{rows:?}");
    let colour = watch.coloured(Duration::ZERO);
    assert!(colour.contains("\x1b[2m1 step · 5s"), "{colour:?}");
}

/// With `--full`, the watch starts each line under a step at its title's column, after its label, and
/// wraps it at the width.
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
    let mut watch = Watch::start(&repo, &["tree", &trail[..8], "--full"]);
    watch.size(24, 60);
    watch.until("fifteen");
    let rows = watch.rows();
    assert!(rows.iter().all(|r| r.chars().count() <= 60), "{rows:?}");
    let at = rows
        .iter()
        .position(|r| r.contains(" done     one "))
        .unwrap_or_else(|| panic!("{rows:?}"));
    let column = rows[at].find("done     ").unwrap() + "done     ".len();
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

/// Every title of a day starts at one column, whatever the count of marks before it.
#[test]
fn watch_starts_each_title_at_one_column() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let changed = add(&repo, &["--under", &trail[..8], "--", "changed part"]);
    let o = cli(
        &repo,
        &[
            "step",
            "done",
            &changed[..8],
            "--changed",
            "more",
            "--",
            "ok",
        ],
    );
    assert!(o.status.success(), "{}", err(&o));
    let plain = add(&repo, &["--under", &trail[..8], "--", "plain part"]);
    finish(&repo, &plain[..8]);
    let mut watch = Watch::start(&repo, &["tree", &trail[..8], "--full"]);
    watch.size(40, 80);
    watch.until("plain part");
    let rows = watch.rows();
    let at = |what: &str| {
        let row = rows.iter().find(|l| l.contains(what)).unwrap();
        row[..row.find(what).unwrap()].chars().count()
    };
    assert_eq!(at("changed part"), at("plain part"), "{rows:?}");
}

/// When `a` takes the selected trail of a past day out of the tree, the watch selects the first line,
/// and the arrows move on from it.
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
    let mut watch = Watch::start(&repo, &["tree"]);
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

/// The key line is cut to the width, so it never wraps and scrolls the screen.
#[test]
fn watch_cuts_the_key_line_to_the_width() {
    let repo = Repo::new();
    add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.size(24, 40);
    watch.until("\x1b[24;1H");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    let keys = last.split("\x1b[24;1H").nth(1).unwrap();
    assert!(keys.chars().count() <= 40, "{keys:?}");
    assert!(keys.ends_with('…'), "{keys:?}");
}

/// A changed done step prints `~` after its mark, and no step prints its place.
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
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.size(24, 80);
    watch.until("1 closed step");
    watch.press("\x1b[B\x1b[B\x1b[B\x1b[C");
    watch.until("✓~ the part");
    let rows = watch.rows();
    let row = rows.iter().find(|r| r.contains("the part")).unwrap();
    assert!(row.ends_with("✓~ the part"), "{rows:?}");
}

/// A revised step prints `↻` after its mark, then `~`, and its working trail keeps the spinner with `↻`
/// after it.
#[test]
fn watch_marks_a_revised_step_and_spins_its_working_trail() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "the part"]);
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
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let next = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--replaces",
            &part[..8],
            "--",
            "next",
        ],
    );
    let o = cli(&repo, &["step", "start", &next[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.size(24, 80);
    watch.until("1 closed step");
    watch.press("\x1b[B\x1b[B\x1b[B\x1b[C");
    watch.until("✓↻~ the part");
    let rows = watch.rows();
    let head = rows.iter().find(|r| r.contains("the trail")).unwrap();
    let spun = head.chars().zip(head.chars().skip(1));
    assert!(
        spun.into_iter().any(|(a, b)| SPIN.contains(a) && b == '↻'),
        "{rows:?}"
    );
}

/// No line of the watch reaches the last column, so clearing to the end of a line never erases its
/// last character.
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
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.size(24, 40);
    watch.until("a working step");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    for row in last.replace("\x1b[24;1H", "\r\n").split("\r\n") {
        assert!(row.chars().count() < 40, "{row:?}");
    }
    let rows = watch.rows();
    let at = rows
        .iter()
        .position(|r| r.contains("a working step"))
        .unwrap();
    assert!(rows[at + 1].ends_with('s'), "{rows:?}");
}

/// On a terminal, `tree` prints the tree again within a second after a record lands in a linked folder that is
/// on.
#[test]
fn watch_prints_the_tree_again_when_a_record_lands_in_a_linked_folder() {
    let repo = Repo::new();
    let o = cli(&repo, &["link"]);
    assert!(o.status.success(), "{}", err(&o));
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
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

/// With `--next`, A closed trail's line whose steps were not all done counts them by mark at its right.
#[test]
fn next_watch_counts_a_closed_trails_steps_by_mark() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 4_000;
    let trail = support::step_note(&repo, &by, None, "the trail", at);
    let part = support::step_note(&repo, &by, Some(&trail), "done part", at);
    support::mark_note(&repo, &by, "done", &part.id, "proof", at + 5);
    let gone = add(&repo, &["--under", &trail.id[..8], "--", "gone part"]);
    let o = cli(&repo, &["step", "drop", &gone[..8], "--", "not needed"]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.size(24, 100);
    watch.until("the trail");
    let rows = watch.rows();
    let row = rows.iter().find(|r| r.contains("the trail")).unwrap();
    assert!(row.ends_with(" ✓ 1 · ✗ 1"), "{rows:?}");
}

/// With `--next`, The watch cuts a long step title and a long trail line's title with `…` to fit the width.
#[test]
fn next_watch_cuts_long_titles_to_the_width() {
    let repo = Repo::new();
    let words = "one two three four five six seven eight nine ten eleven twelve";
    let (by, at) = (support::account(), support::now() - 3_600);
    let title = format!("done trail {words}");
    let done = support::step_note(&repo, &by, None, &title, at);
    let part = support::step_note(&repo, &by, Some(&done), "its part", at + 1);
    support::mark_note(&repo, &by, "done", &part.id, "proof", at + 2);
    let trail = add(&repo, &["--", "the trail"]);
    add(
        &repo,
        &["--under", &trail[..8], "--", &format!("open step {words}")],
    );
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.size(24, 40);
    watch.until("open step");
    watch.press("\x1b[B\x1b[B\x1b[B\r");
    watch.until("done trail");
    let rows = watch.rows();
    assert!(rows.iter().all(|r| r.chars().count() <= 40), "{rows:?}");
    for title in ["done trail one", "open step one"] {
        let row = rows.iter().find(|r| r.contains(title));
        assert!(row.is_some_and(|r| r.contains('…')), "{rows:?}");
    }
    assert!(!rows.join("\n").contains("twelve"), "{rows:?}");
}

/// With `--next`, With no id, the watch heads today with a line with 🗓️, the day and its counts, groups the trails by
/// project under a 📁 heading, each trail's mark under the project's name, its active trails first, then lists the five earlier days folded on one line each, and
/// only with more days past them a grey `Show more`, which `enter` turns into the rest of the days folded and
/// `Show less`, the selection on the first day it added.
#[test]
fn next_watch_groups_today_by_project_and_shows_more() {
    let repo = Repo::new();
    for n in 1..=6 {
        let at = support::now() - n * support::DAY;
        support::step_note(&repo, &support::account(), None, &format!("trail {n}"), at);
    }
    add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.size(30, 80);
    watch.until("▸   Show more");
    let rows = watch.rows();
    assert!(
        rows[0].starts_with("  🗓\u{fe0f}  ") && rows[0].ends_with(" · 1 active · 0 done"),
        "{rows:?}"
    );
    assert_eq!(rows[1..4], ["  ", "      📁  repo", "  "], "{rows:?}");
    assert_eq!(
        support::watch::still(&rows[4]),
        "          ⠿  the trail",
        "{rows:?}"
    );
    let days = |rows: &[String]| {
        let folded = rows.iter().filter(|r| r.contains(", ") && r.contains("▸ "));
        folded.count()
    };
    assert_eq!(days(&rows), 5, "{rows:?}");
    assert!(!rows.join("\n").contains("trail 1"), "{rows:?}");
    watch.press(&"\x1b[B".repeat(7));
    watch.until("▌ ▸   Show more");
    watch.press("\r");
    watch.until("▾   Show less");
    let rows = watch.rows();
    assert_eq!(days(&rows), 6, "{rows:?}");
    let picked = rows.iter().find(|r| r.starts_with('▌')).unwrap();
    assert!(picked.contains(", "), "{rows:?}");
}

/// With `--next`, A row many levels deep cuts its title with `…` within the width.
#[test]
fn next_watch_cuts_a_deep_row_to_the_width() {
    let repo = Repo::new();
    let mut under = add(&repo, &["--", "the trail"]);
    for n in 1..=8 {
        under = add(
            &repo,
            &["--under", &under[..8], "--", &format!("level {n}")],
        );
    }
    let words = "alpha bravo charlie delta echo foxtrot golf hotel";
    add(
        &repo,
        &["--under", &under[..8], "--", &format!("deep {words}")],
    );
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.size(40, 40);
    watch.until("level 1");
    watch.press("\x1b[B*");
    watch.until("level 4");
    watch.trees(Duration::from_millis(300));
    let rows = watch.rows();
    assert!(rows.iter().all(|r| r.chars().count() <= 40), "{rows:?}");
    assert!(!rows.join("\n").contains(words), "{rows:?}");
    assert!(rows.iter().any(|r| r.ends_with('…')), "{rows:?}");
}

/// With `--next`, A trail with more than five closed steps folds them into one `✓ <n> closed` line before the rest, its
/// expired ones among them, and opens it to them.
#[test]
fn next_watch_holds_expired_steps_in_the_closed_line() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 2 * 3600;
    let trail = support::step_note(&repo, &by, None, "the trail", at);
    for n in 1..=5 {
        let part = format!("done part {n}");
        let done = support::step_note(&repo, &by, Some(&trail), &part, at + n);
        support::mark_note(&repo, &by, "done", &done.id, "proof", at + n);
    }
    support::step_note(&repo, &by, Some(&trail), "left part", at + 9);
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail.id[..8]]);
    watch.size(24, 80);
    watch.until("6 closed");
    let rows = watch.rows();
    assert_eq!(rows[1], "    └▸ ✓  6 closed", "{rows:?}");
    assert!(rows[0].ends_with("✓ 5 · ◌ 1"), "{rows:?}");
    assert!(!rows.join("\n").contains("left part"), "{rows:?}");
    watch.press("\x1b[B\x1b[B\x1b[C");
    watch.until("left part");
    assert!(
        watch.rows().contains(&"       └─ ◌  left part".to_string()),
        "{:?}",
        watch.rows()
    );
}

/// With `--next`, No line of the watch reaches the last column, so clearing to the end of a line never erases its
/// last character.
#[test]
fn next_watch_leaves_the_last_column_empty() {
    let repo = Repo::new();
    let trail = add(
        &repo,
        &["--", "a trail whose title runs well past forty characters"],
    );
    let step = add(&repo, &["--under", &trail[..8], "--", "a working step"]);
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.size(24, 40);
    watch.until("a working step");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    for row in last.replace("\x1b[24;1H", "\r\n").split("\r\n") {
        assert!(row.chars().count() < 40, "{row:?}");
    }
    let rows = watch.rows();
    let at = rows
        .iter()
        .position(|r| r.contains("a trail whose"))
        .unwrap_or_else(|| panic!("{rows:?}"));
    assert!(
        rows[at + 1].ends_with("runs well past forty") && rows[at + 2].ends_with("characters"),
        "{rows:?}"
    );
}

/// With `--next`, The watch draws a block with its project and done/all at the right of its title, its steps on tree
/// lines under it, a step with steps under it closed behind `▸` and spinning while one under it works, its rules
/// folded to one line, no ids and no places, in colour: § magenta, the tree lines grey whatever the state.
#[test]
fn next_watch_draws_a_block_with_tree_lines_and_colour_anchors() {
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
    let fix = add(
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
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.size(24, 80);
    watch.until("the fix");
    let rows: Vec<String> = watch
        .rows()
        .iter()
        .map(|r| support::watch::still(r))
        .collect();
    assert!(
        rows[0].starts_with("  ▾ ⠿  the trail") && rows[0].ends_with(" repo · 0/3"),
        "{rows:?}"
    );
    assert!(
        rows[1].starts_with("    ├▸ ⠿  done part ") && rows[1].ends_with("0/1"),
        "{rows:?}"
    );
    assert_eq!(rows[2], "    ├─ ○  the fix", "{rows:?}");
    assert_eq!(rows[3], "    └▸ §  1 rule", "{rows:?}");
    let all = rows.join("\n");
    for gone in ["prices.py", "deeper part", &trail[..8], &fix[..8]] {
        assert!(!all.contains(gone), "{gone:?} in {rows:?}");
    }
    let raw = watch.coloured(Duration::ZERO);
    for anchor in ["\x1b[38;2;192;96;255m§", "\x1b[38;2;44;66;104m├"] {
        assert!(raw.contains(anchor), "{anchor:?} in {raw:?}");
    }
}

/// With `--next`, A trail lists its closed steps among the rest in the order they came; a done step with steps under it
/// stays closed behind `▸`, done/all at its right, until it opens to them.
#[test]
fn next_watch_folds_closed_steps_and_done_branches() {
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
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.size(24, 80);
    watch.until("next");
    let rows = watch.rows();
    assert!(
        rows[1].starts_with("    ├▸ ✓  parent ") && rows[1].ends_with("1/1"),
        "{rows:?}"
    );
    assert_eq!(rows[2..4], ["    ├─ ✗  gone", "    └─ ○  next"], "{rows:?}");
    let all = rows.join("\n");
    assert!(
        !all.contains("child") && !all.contains("not needed"),
        "{rows:?}"
    );
    watch.press("\x1b[B\x1b[B\x1b[C");
    watch.until("child");
    assert!(
        watch.rows().contains(&"    │  └─ ✓  child".to_string()),
        "{:?}",
        watch.rows()
    );
}

/// With `--next`, A project's closed trails fold into one line that opens to a line a trail, each its steps counted by
/// state dim at the right of its title, with no time and no count of its rules.
#[test]
fn next_watch_prints_a_trail_lines_details_under_its_title() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 4_000;
    let one = support::step_note(&repo, &by, None, "first", at);
    let part = support::step_note(&repo, &by, Some(&one), "part", at);
    support::mark_note(&repo, &by, "done", &part.id, "proof", at + 5);
    let many = support::step_note(&repo, &by, None, "second", at + 10);
    for n in 0..12 {
        let part = support::step_note(&repo, &by, Some(&many), &format!("part {n}"), at + 10);
        support::mark_note(&repo, &by, "done", &part.id, "proof", at + 20);
    }
    let project = repo.project_id();
    for n in 0..10 {
        let scope = format!("trail:{}", one.id);
        support::stored_rule(
            &repo,
            (&project, "repo"),
            &by,
            (Some(&scope), None),
            &format!("rule {n}"),
            at,
        );
    }
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.size(24, 100);
    watch.until("2 closed");
    assert!(!watch.rows().join("\n").contains("first"));
    watch.press("\x1b[B\r");
    watch.until("second");
    let rows = watch.rows();
    let row = |what: &str| rows.iter().find(|l| l.contains(what)).unwrap();
    let first = row("first");
    assert!(
        first.starts_with("           ▸ ✓  first ") && first.ends_with(" ✓ 1"),
        "{rows:?}"
    );
    let second = row("second");
    assert!(
        second.starts_with("           ▸ ✓  second ") && second.ends_with(" ✓ 12"),
        "{rows:?}"
    );
    assert!(!rows.join("\n").contains("rule"), "{rows:?}");
    let colour = watch.coloured(Duration::ZERO);
    assert!(colour.contains("\x1b[2m 12"), "{colour:?}");
}

/// With `--next`, A block prints its steps in the order they came, closed ones among them, then its rules line; a step
/// with steps under it stays closed behind `▸` until it opens to them in their order.
#[test]
fn next_watch_prints_open_steps_then_rules_then_closed_steps() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 60;
    let trail = support::step_note(&repo, &by, None, "the trail", at);
    let scope = format!("trail:{}", trail.id);
    let project = (repo.project_id(), "repo");
    let rule = (Some(scope.as_str()), None);
    support::stored_rule(&repo, (&project.0, project.1), &by, rule, "a rule", at);
    let finished = support::step_note(&repo, &by, Some(&trail), "finished", at + 1);
    support::mark_note(&repo, &by, "done", &finished.id, "proof", at + 2);
    let parent = support::step_note(&repo, &by, Some(&trail), "parent", at + 3);
    let old = support::step_note(&repo, &by, Some(&parent), "old child", at + 4);
    support::mark_note(&repo, &by, "done", &old.id, "proof", at + 5);
    support::step_note(&repo, &by, Some(&parent), "new child", at + 6);
    support::step_note(&repo, &by, Some(&trail), "newest", at + 7);
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail.id[..8]]);
    watch.size(24, 80);
    watch.until("newest");
    let tree = |rows: Vec<String>| -> Vec<String> {
        rows.iter()
            .skip(1)
            .map(|r| r.trim_end().to_string())
            .collect()
    };
    let rows = tree(watch.rows());
    let starts = [
        "    ├─ ✓  finished",
        "    ├▸ ○  parent",
        "    ├─ ○  newest",
        "    └▸ §  1 rule",
    ];
    assert_eq!(rows.len(), starts.len(), "{rows:?}");
    for (row, start) in rows.iter().zip(starts) {
        assert!(row.starts_with(start), "{start:?} in {rows:?}");
    }
    assert!(rows[1].ends_with("1/2"), "{rows:?}");
    watch.press("\x1b[B\x1b[B\x1b[B\x1b[C");
    watch.until("new child");
    let rows = tree(watch.rows());
    assert_eq!(
        rows[2..4],
        ["    │  ├─ ✓  old child", "    │  └─ ○  new child"],
        "{rows:?}"
    );
}

/// With `--next`, When `a` takes the selected earlier day out of the tree, the watch selects the first line, and the
/// arrows move on from it to the first folded day.
#[test]
fn next_watch_keeps_a_selection_when_its_line_leaves() {
    let repo = Repo::new();
    for n in 1..=6 {
        let at = support::now() - n * support::DAY;
        support::step_note(&repo, &support::account(), None, &format!("trail {n}"), at);
    }
    add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.size(32, 80);
    watch.until("▸   Show more");
    watch.press("a");
    watch.until("▾   Show less");
    watch.press(&"\x1b[B".repeat(7));
    std::thread::sleep(Duration::from_millis(300));
    watch.press("a");
    watch.until_spun("▌         ⠿  the trail");
    watch.press("\x1b[B");
    watch.until("▌ ▸ ");
}

/// With `--next`, A changed done step prints `˜` after its mark among the trail's steps, its title in the same column as
/// the rest, and no step prints its place.
#[test]
fn next_watch_marks_a_change_and_prints_no_place() {
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
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.size(24, 80);
    watch.until("✓˜ the part");
    let rows = watch.rows();
    let column = |text: &str| {
        let row = rows.iter().find(|r| r.contains(text)).unwrap();
        row[..row.find(text).unwrap()].chars().count()
    };
    assert_eq!(column(" next"), column(" the part"), "{rows:?}");
    assert!(
        rows.iter().any(|r| r.ends_with("─ ✓˜ the part")),
        "{rows:?}"
    );
    assert!(!rows.join("\n").contains("shelf/a.py"), "{rows:?}");
}

/// With `--next`, A run of `↓` typed at once moves the selection by each key, and draws one tree for the run, not one
/// for each key.
#[test]
fn next_watch_draws_a_run_of_keys_as_one_tree() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    for n in 1..=12 {
        add(
            &repo,
            &["--under", &trail[..8], "--", &format!("step {n:02}")],
        );
    }
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.until("step 01");
    let before = watch.trees(Duration::from_millis(300)).len();
    let tenth = watch.rows()[9].clone();
    let title = tenth.find("step ").unwrap() + "step 00".len();
    watch.press(&"\x1b[B".repeat(10));
    watch.until(&format!("▌{}", &tenth[1..title]));
    let trees = watch.trees(Duration::from_millis(300));
    let picks: Vec<Option<usize>> = trees[before..]
        .iter()
        .map(|t| t.split("\r\n").position(|r| r.starts_with('▌')))
        .collect();
    let moves = picks.windows(2).filter(|w| w[0] != w[1]).count();
    assert!(moves < 4, "{moves} moves of the selection for 10 keys");
}

/// With `--next`, A step whose newest record is less than 2 seconds old prints its title bold, then in normal weight;
/// a done step's mark is green.
#[test]
fn next_watch_prints_a_fresh_step_bold() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let done = add(&repo, &["--under", &trail[..8], "--", "a done part"]);
    finish(&repo, &done[..8]);
    add(&repo, &["--under", &trail[..8], "--", "the part"]);
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.size(24, 80);
    watch.until("the part");
    assert!(watch.coloured(Duration::ZERO).contains("\x1b[1mthe part"));
    std::thread::sleep(Duration::from_millis(2500));
    let raw = watch.coloured(Duration::from_millis(300));
    assert!(!raw.contains("\x1b[1mthe part"), "{raw:?}");
    assert!(raw.contains("\x1b[38;2;52;230;140m✓\x1b[0m  "), "{raw:?}");
}

/// With `--next`, On a terminal, `tree` prints a tree whose times stand still again, wrapped to the new width, when the
/// terminal's width changes: an open trail wraps its whole title.
#[test]
fn next_watch_on_a_terminal_prints_the_tree_again_when_its_width_changes() {
    let repo = Repo::new();
    let hours = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - 7_200;
    let text = "a trail whose words run well past thirty characters";
    let trail = support::step_note(&repo, &support::account(), None, text, hours).id;
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.until("thirty characters");
    watch.resize(30);
    let start = std::time::Instant::now();
    let wrapped = |rows: &[String]| rows.iter().any(|r| r.trim().starts_with("characters"));
    while !wrapped(&watch.rows()) && start.elapsed() < Duration::from_secs(5) {
        watch.trees(Duration::from_millis(50));
    }
    assert!(wrapped(&watch.rows()), "{:?}", watch.rows());
    let trees = watch.trees(Duration::ZERO);
    assert!(trees.len() >= 2, "{trees:?}");
}

/// With `--next`, `↓` selects a step of a block, `c` copies its id's first 8 characters with `pbcopy`, `m` its table as
/// Markdown, and each says so; its proof shows only with `f`, full.
#[test]
fn next_watch_opens_a_step_and_copies_its_id() {
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
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.until("the part");
    watch.press("\x1b[B\x1b[B");
    watch.until("\r\n▌   └─ ✓  the part");
    watch.until(&format!("c copy {}", &part[..8]));
    assert!(!watch.rows().join("\n").contains("it works"));
    watch.press("c");
    watch.until(&format!("copied {}", &part[..8]));
    let copied = std::fs::read_to_string(repo.home().join("copied")).unwrap();
    assert_eq!(copied, &part[..8]);
    watch.press("m");
    watch.until("copied markdown");
    let copied = std::fs::read_to_string(repo.home().join("copied")).unwrap();
    assert!(
        copied.starts_with("| | Step | Took | Proof |\n") && copied.contains(" | it works |"),
        "{copied}"
    );
    watch.press("f");
    watch.until("done     it works");
}

/// With `--next`, A block folds its rules to one line, and `→` opens it to a line a rule, each wrapped whole to fit,
/// with no scope or id.
#[test]
fn next_watch_opens_the_rules_line() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    add(&repo, &["--under", &trail[..8], "--", "the part"]);
    let rule = "When you price a senior: use cents, because floats lost a cent on the receipt.";
    let scope = format!("trail:{}", &trail[..8]);
    let o = cli(&repo, &["decide", "--scope", &scope, "--", rule]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.size(24, 80);
    watch.until("└▸ §  1 rule");
    let tree = watch.rows().join("\n");
    assert!(
        !tree.contains("When you price") && !tree.contains(&trail[..8]),
        "{tree:?}"
    );
    watch.press("\x1b[B\x1b[B\x1b[B");
    watch.until("\r\n▌   └▸ §  1 rule");
    watch.press("\x1b[C");
    watch.until("└▾ §  1 rule");
    watch.until("       └─ §  When you price a senior");
    watch.until("the receipt.");
    assert!(
        !watch.rows().join("\n").contains(&scope),
        "{:?}",
        watch.rows()
    );
}

/// With `--next`, A step with steps under it stays closed behind `▸`; `*` opens it and every step under it and closes
/// them all again; `→` opens it, `←` on a step under it selects it, and `←` closes it.
#[test]
fn next_watch_opens_and_closes_with_arrows_and_star() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let parent = add(&repo, &["--under", &trail[..8], "--", "parent"]);
    let child = add(&repo, &["--under", &parent[..8], "--", "child"]);
    add(&repo, &["--under", &child[..8], "--", "grandchild"]);
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.until("parent");
    assert!(!watch.rows().join("\n").contains("child"));
    watch.press("\x1b[B\x1b[B");
    watch.until("▌   └▸ ○  parent");
    watch.press("*");
    watch.until("grandchild");
    watch.press("*");
    watch.without("child");
    watch.press("\x1b[C");
    watch.until("└▸ ○  child");
    assert!(!watch.rows().join("\n").contains("grandchild"));
    watch.press("\x1b[B");
    watch.until("▌      └▸ ○  child");
    watch.press("\x1b[D");
    watch.until("▌   └▾ ○  parent");
    watch.press("\x1b[D");
    watch.without("child");
}

/// With `--next`, A revised done step keeps its `˜` with no `revised by` at its right, and its working trail keeps the
/// spinner with `↻` after it.
#[test]
fn next_watch_marks_a_revised_step_and_spins_its_working_trail() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "the part"]);
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
    std::thread::sleep(std::time::Duration::from_millis(1100));
    let next = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--replaces",
            &part[..8],
            "--",
            "next",
        ],
    );
    let o = cli(&repo, &["step", "start", &next[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.size(24, 100);
    watch.until("✓˜ the part");
    let rows = watch.rows();
    assert!(!rows.join("\n").contains("revised by"), "{rows:?}");
    let head = rows.iter().find(|r| r.contains("the trail")).unwrap();
    let spun = head.chars().zip(head.chars().skip(1));
    assert!(
        spun.into_iter().any(|(a, b)| SPIN.contains(a) && b == '↻'),
        "{rows:?}"
    );
}

/// With `--next`, A trail begun days ago with a record in the last half hour is a live session: today shows it as a block
/// with the spinner for its mark, though its steps are all done, and an idle trail of that day stays in the day's folded
/// line under today.
#[test]
fn next_watch_shows_a_live_session_on_top_in_all() {
    let repo = Repo::new();
    let by = support::account();
    let old = support::now() - 2 * support::DAY;
    support::step_note(&repo, &by, None, "an old idle trail", old);
    let trail = support::step_note(&repo, &by, None, "an old live trail", old + 60);
    let part = support::step_note(&repo, &by, Some(&trail), "its part", old + 61);
    support::mark_note(&repo, &by, "done", &part.id, "proof", support::now() - 60);
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.size(40, 80);
    watch.until("an old live trail");
    assert!(!watch.rows().join("\n").contains("an old idle trail"));
    watch.press("\x1b[B\x1b[B\x1b[B\r");
    watch.until("an old idle trail");
    let rows = watch.rows();
    let live = rows
        .iter()
        .position(|r| r.contains("an old live trail"))
        .unwrap();
    let idle = rows
        .iter()
        .position(|r| r.contains("an old idle trail"))
        .unwrap();
    assert!(live < idle, "{rows:?}");
    assert!(
        support::watch::still(&rows[live]).starts_with("        ▾ ⠿  an old live trail"),
        "{rows:?}"
    );
    assert!(rows[0].ends_with(" · 1 active · 0 done"), "{rows:?}");
}

/// With `--next`, `↓` selects a line with a `▌` in the cursor column, `enter` opens the fold of a project's closed
/// trails, the selected trail's title prints bold, and `enter` opens a done trail into a block and closes it again.
#[test]
fn next_watch_selects_a_trail_and_opens_it_with_enter() {
    let repo = Repo::new();
    let (by, at) = (support::account(), support::now() - 3_600);
    let ended = support::step_note(&repo, &by, None, "an ended trail", at);
    let part = support::step_note(&repo, &by, Some(&ended), "its part", at + 1);
    support::mark_note(&repo, &by, "done", &part.id, "proof", at + 2);
    add(&repo, &["--", "an active trail"]);
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.until("1 closed");
    watch.press("\x1b[B\x1b[B");
    watch.until("\r\n▌       ▸ ✓  1 closed");
    watch.press("\r");
    watch.until("an ended trail");
    watch.press("\x1b[B");
    watch.until("\r\n▌          ▸ ✓  an ended trail");
    std::thread::sleep(Duration::from_millis(300));
    let raw = watch.coloured(Duration::ZERO);
    assert!(raw.contains("\x1b[1man ended trail"), "{raw:?}");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    assert!(
        support::watch::still(&last).contains("\r\n          ⠿  an active trail"),
        "{last:?}"
    );
    watch.press("\r");
    watch.until("its part");
    watch.press("\r");
    watch.without("its part");
}

/// With `--next`, A watch spins a working step's mark, and puts done/all of its trail's steps at the right of the
/// trail's title.
#[test]
fn next_watch_spins_a_working_step_and_says_when_the_trail_was_active() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let step = add(&repo, &["--under", &trail[..8], "--", "the step"]);
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.until("the step");
    let trees = watch.trees(Duration::from_millis(1000));
    let marks: std::collections::BTreeSet<char> = trees
        .iter()
        .flat_map(|t| t.chars())
        .filter(|c| SPIN.contains(*c))
        .collect();
    assert!(marks.len() >= 3, "{trees:?}");
    let last = trees.last().unwrap();
    assert!(last.contains(" repo · 0/1"), "{last:?}");
}

/// With `--next`, On a terminal, `tree <id>` of a trail whose times stand still prints it again only when a record
/// changes the tree.
#[test]
fn next_watch_prints_the_tree_again_only_when_it_changes() {
    let repo = Repo::new();
    let (by, at) = (support::account(), support::now() - 3_600);
    let trail = support::step_note(&repo, &by, None, "the trail", at).id;
    support::mark_note(&repo, &by, "done", &trail, "proof", at + 1);
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.until("✓  the trail");
    let shown = watch.trees(Duration::from_millis(1200)).len();
    let o = cli(
        &repo,
        &["decide", "--", "When you add a page: copy the frame."],
    );
    assert!(o.status.success(), "{}", err(&o));
    assert_eq!(watch.trees(Duration::from_millis(1500)).len(), shown);
}

/// With `--next`, A scene that fits only once the window widens shows still at once: the walker already cheering at
/// the summit, no climb.
#[test]
fn next_watch_shows_a_scene_still_when_the_window_widens() {
    let repo = Repo::new();
    add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.size(40, 80);
    watch.until("the trail");
    watch.size(40, 140);
    watch.until("\\^o^/ |>");
    assert!(!watch.shown(Duration::ZERO).contains("•_•"));
}

/// With `--next`, A watch prints its keys on the screen's last row, and scrolls a tree taller than the rows above
/// them, with `↑ <n> more` and `↓ <n> more` rows; `G`, `g` and `PgDn` scroll it.
#[test]
fn next_watch_scrolls_a_tree_taller_than_the_screen() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    for n in 0..20 {
        add(
            &repo,
            &["--under", &trail[..8], "--", &format!("step {n:02}")],
        );
    }
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.until("the trail");
    watch.size(10, 80);
    watch.until("↓ 13 more");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    assert!(last.contains("\x1b[10;1H↑↓ select"), "{last:?}");
    let rows: Vec<&str> = last
        .split("\x1b[10;1H")
        .next()
        .unwrap()
        .split("\r\n")
        .collect();
    assert_eq!(rows.len(), 9, "{last:?}");
    assert_eq!(rows[8], "  ↓ 13 more", "{last:?}");
    watch.press("G");
    watch.until("↑ 13 more");
    assert!(!watch.trees(Duration::ZERO).pop().unwrap().contains("  ↓ "));
    watch.press("g");
    watch.until("↓ 13 more");
    watch.press("\x1b[6~");
    watch.until("↑ 8 more");
}

/// With `--next`, On a terminal, `tree` with no id says no trail is there and names `tree --all`, then prints what `tree`
/// prints: the trails of every project.
#[test]
fn next_watch_with_no_id_prints_what_tree_prints() {
    let repo = Repo::new();
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.until("No active trails");
    watch.until("celis tree --all");
    let first = add(&repo, &["--", "the first trail"]);
    watch.until("the first trail");
    let shop = repo.other("shop");
    let o = repo.run(&shop, &["step", "add", "--", "the shop trail"], &[], "");
    assert!(o.status.success(), "{}", err(&o));
    watch.until("the shop trail");
    finish(&repo, &first[..8]);
    watch.until_spun("⠿  the first trail");
}

/// With `--next`, a trail keeps the blank row after it and the same meta at its right whether it is open
/// or closed, and shows its steps under it with no expect line.
#[test]
fn next_watch_keeps_a_trails_rows_still_as_it_opens_and_closes() {
    let repo = Repo::new();
    let first = add(
        &repo,
        &["--new", "--", "the first trail\nexpect: it reads well"],
    );
    add(&repo, &["--under", &first[..8], "--", "its step"]);
    std::thread::sleep(Duration::from_millis(1100));
    add(&repo, &["--new", "--", "the second trail"]);
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.size(30, 80);
    watch.until("its step");
    let rows = watch.rows();
    let column = |row: &str, text: &str| row[..row.find(text).unwrap()].chars().count();
    let at = rows
        .iter()
        .position(|r| r.contains("the first trail"))
        .unwrap();
    assert!(!rows.join("\n").contains("expect"), "{rows:?}");
    assert_eq!(rows[at + 1], "          └─ ○  its step", "{rows:?}");
    assert_eq!(column(&rows[at], "the first trail"), 13, "{rows:?}");
    let meta = |row: &str| {
        row.split("the first trail")
            .nth(1)
            .unwrap()
            .trim()
            .to_string()
    };
    let open = meta(&rows[at]);
    watch.press("\x1b[B");
    watch.until_spun("▌         ⠿  the second trail");
    watch.press("\x1b[B");
    watch.until_spun("▌       ▾ ⠿  the first trail");
    watch.press("\x1b[D");
    watch.until_spun("▌       ▸ ⠿  the first trail");
    let rows = watch.rows();
    let closed = rows
        .iter()
        .position(|r| r.contains("the first trail"))
        .unwrap();
    assert_eq!(closed, at, "{rows:?}");
    assert!(rows[at - 1].trim().is_empty(), "{rows:?}");
    assert_eq!(meta(&rows[at]), open, "{rows:?}");
}

/// With `--next`, The opening scene plays the first time a watch has room for it, and never again: a later
/// watch shows the card and the tree with no scene.
#[test]
fn next_watch_plays_the_opening_scene_once() {
    let repo = Repo::new();
    add(&repo, &["--", "the trail"]);
    let mut first = Watch::start(&repo, &["tree", "--next"]);
    first.size(40, 140);
    first.until("dead end");
    first.press("q");
    let _ = first.child.wait();
    assert!(repo.home().join("opening").exists());
    let mut again = Watch::start(&repo, &["tree", "--next"]);
    again.size(40, 140);
    again.until("the trail");
    again.trees(Duration::from_millis(1500));
    assert!(!again.shown(Duration::ZERO).contains("dead end"));
}

/// With `--next`, The watch draws only its trails: no card of what celis says above them.
#[test]
fn next_watch_draws_only_its_trails() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let scope = format!("trail:{}", &trail[..8]);
    let rule = "When you add a page: copy the frame, because it holds the header.";
    let o = cli(&repo, &["decide", "--scope", &scope, "--", rule]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &["tree", "--next"]);
    watch.size(24, 80);
    watch.until("the trail");
    watch.trees(Duration::from_millis(1000));
    let shown = watch.shown(Duration::ZERO);
    for card in ["This trail so far", "Learned", "copy the frame"] {
        assert!(!shown.contains(card), "{card:?} in {shown:?}");
    }
    let rows = watch.screen_rows();
    assert!(rows[0].ends_with(" · 1 active · 0 done"), "{rows:?}");
}

/// With `--next`, An open trail whose title wraps carries its tree line down from its mark through the wrapped rows to its
/// first step.
#[test]
fn next_watch_carries_the_line_down_a_wrapped_title() {
    let repo = Repo::new();
    let title = "a trail whose title runs well past the width of this screen";
    let trail = add(&repo, &["--", title]);
    add(&repo, &["--under", &trail[..8], "--", "its step"]);
    let mut watch = Watch::start(&repo, &["tree", "--next", &trail[..8]]);
    watch.size(24, 40);
    watch.until("its step");
    let rows = watch.rows();
    let step = rows.iter().position(|r| r.contains("its step")).unwrap();
    assert!(step >= 2, "{rows:?}");
    for row in &rows[1..step] {
        assert!(row.starts_with("    │  "), "{rows:?}");
    }
    assert!(rows[step].starts_with("    └─ "), "{rows:?}");
}
