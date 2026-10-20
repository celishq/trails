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
    watch.until("✓  the trail");
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
    watch.until("✓  the first trail");
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
/// terminal's width changes: an open trail wraps its whole title.
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
    watch.until("│  words run well");
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

/// A watch spins a working step's mark, and puts a bar of its trail's steps with done/all and its worked
/// time at the right of the trail's title.
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
    assert!(last.contains("━  0/1 · "), "{last:?}");
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
    watch.until("↓ 14 more");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    assert!(last.contains("\x1b[10;1H↑↓ select"), "{last:?}");
    let rows: Vec<&str> = last
        .split("\x1b[10;1H")
        .next()
        .unwrap()
        .split("\r\n")
        .collect();
    assert_eq!(rows.len(), 9, "{last:?}");
    assert_eq!(rows[8], "  ↓ 14 more", "{last:?}");
    watch.press("G");
    watch.until("↑ 14 more");
    assert!(!watch.trees(Duration::ZERO).pop().unwrap().contains("  ↓ "));
    watch.press("g");
    watch.until("↓ 14 more");
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
    assert!(
        raw.contains("\r\n▌ \x1b[38;2;30;48;76m│ \x1b[0m\x1b[2m▸\x1b[0m"),
        "{raw:?}"
    );
    assert!(raw.contains("\x1b[1man ended trail"), "{raw:?}");
    let last = watch.trees(Duration::ZERO).pop().unwrap();
    assert!(last.contains("\r\n▌ │ ▸ ✓  an ended trail"), "{last:?}");
    assert!(last.contains("\r\n  │ ▾ ○  an active trail"), "{last:?}");
    watch.press("\r");
    watch.until("▸ ✓  1 closed step");
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
    let tenth = watch.rows()[10].clone();
    let title = tenth.find("step ").unwrap() + "step 00".len();
    watch.press(&"\x1b[B".repeat(10));
    watch.until(&format!("▌{}", &tenth[1..title]));
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
        assert!(
            (1..=8).all(|n| !tick.contains(&format!("step {n}"))),
            "{tick:?}"
        );
    }
}

/// A row many levels deep cuts its title with `…` within the width.
#[test]
fn watch_cuts_a_deep_row_to_the_width() {
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
    watch.until("level 1");
    let rows = watch.rows();
    assert!(rows.iter().all(|r| r.chars().count() <= 40), "{rows:?}");
    assert!(!rows.join("\n").contains(words), "{rows:?}");
    assert!(rows.iter().any(|r| r.ends_with('…')), "{rows:?}");
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
    watch.until("\r\n▌   └─ ▸ ✓  1 closed step");
    watch.press("\x1b[C\x1b[B");
    watch.until("\r\n▌        └─ ▸ ✓  the part");
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

/// A block folds its rules to one line, and `→` opens it to a line a rule, each cut with `…` to fit,
/// its scope and id at its right.
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
    watch.size(24, 80);
    watch.until("└─ ▸ §  1 rule");
    let tree = watch.rows().join("\n");
    assert!(
        !tree.contains("When you price") && !tree.contains(&trail[..8]),
        "{tree:?}"
    );
    watch.press("\x1b[B\x1b[B\x1b[B");
    watch.until("\r\n▌   └─ ▸ §  1 rule");
    watch.press("\x1b[C");
    watch.until("└─ ▾ §  1 rule");
    watch.until("       └─── §  When you price a");
    watch.until(&format!("  {scope} · "));
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
    watch.until("▌   └─ ▾ ○  parent");
    watch.press("*");
    watch.until("parent text");
    watch.until("child text");
    watch.press("*");
    watch.without("parent text");
    watch.without("child text");
    watch.press("\x1b[B");
    watch.until("▌        └─ ▸ ○  child");
    watch.press("\x1b[D");
    watch.until("▌   └─ ▾ ○  parent");
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

/// The watch draws a block with its project, a bar of its steps and done/all at the right of its title,
/// a rail from its mark to its tree lines, its rules folded to one line, each step's id at its right and
/// no places, in colour: § magenta, the lanes of a working trail the spinner's cyan.
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
    let mut watch = Watch::start(&repo, &["tree", &trail[..8]]);
    watch.size(24, 80);
    watch.until("the fix");
    let rows = watch.rows();
    assert!(
        rows[0].starts_with("  ▾ ")
            && rows[0].contains(" repo · ━")
            && rows[0].contains("  0/3 · "),
        "{rows:?}"
    );
    assert_eq!(rows[1], "    │", "{rows:?}");
    assert!(
        rows[2].starts_with("    ├─── ○  the fix ") && rows[2].ends_with(&fix[..8]),
        "{rows:?}"
    );
    assert!(
        rows[3].starts_with("    ├─ ▾ ○  done part ") && rows[3].contains(" 0/1 · "),
        "{rows:?}"
    );
    assert!(
        rows[4].starts_with("    │    └─── ") && rows[4].ends_with('s'),
        "{rows:?}"
    );
    assert!(!rows.join("\n").contains("prices.py"), "{rows:?}");
    assert_eq!(rows[5], "    └─ ▸ §  1 rule", "{rows:?}");
    assert!(!rows.join("\n").contains(&trail[..8]), "{rows:?}");
    let raw = watch.coloured(Duration::ZERO);
    for anchor in ["\x1b[38;2;192;96;255m§", "\x1b[38;2;34;211;238m│"] {
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
    assert!(
        raw.contains("\x1b[38;2;52;230;140m✓\x1b[0m  1 closed step"),
        "{raw:?}"
    );
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
    support::done_note(&repo, &by, &finished, at + 2);
    let parent = support::step_note(&repo, &by, Some(&trail), "parent", at + 3);
    let old = support::step_note(&repo, &by, Some(&parent), "old child", at + 4);
    support::done_note(&repo, &by, &old, at + 5);
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
        "    ├─── ○  newest",
        "    ├─ ▾ ○  parent",
        "    │    ├─── ○  new child",
        "    │    └─ ▸ ✓  1 closed step",
        "    ├─ ▸ §  1 rule",
        "    └─ ▸ ✓  1 closed step",
    ];
    assert_eq!(tree.len(), starts.len(), "{rows:?}");
    for (row, start) in tree.iter().zip(starts) {
        assert!(row.starts_with(start), "{start:?} in {rows:?}");
    }
}

/// The closed line holds a trail's expired steps too, and counts them by mark at its right: `✓`, `✗`,
/// then `◌`.
#[test]
fn watch_holds_expired_steps_in_the_closed_line() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 2 * 3600;
    let trail = support::step_note(&repo, &by, None, "the trail", at);
    let done = support::step_note(&repo, &by, Some(&trail), "done part", at + 1);
    support::done_note(&repo, &by, &done, at + 2);
    support::step_note(&repo, &by, Some(&trail), "left part", at + 3);
    let mut watch = Watch::start(&repo, &["tree", &trail.id[..8]]);
    watch.size(24, 80);
    watch.until("2 closed steps");
    let rows = watch.rows();
    let closed = rows.iter().find(|r| r.contains("└─ ▸ ✓  2 closed steps"));
    assert!(
        closed.is_some_and(|r| r.ends_with(" ✓ 1 · ◌ 1")),
        "{rows:?}"
    );
    assert!(!rows.join("\n").contains("left part"), "{rows:?}");
}

/// The closed line holds the done and dropped steps, counted `✓ 1 · ✗ 1` at its right; open, a done step
/// with steps under it prints its own closed line only when it is open, and a bar of them at its right.
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
    let closed = rows.iter().find(|r| r.contains("└─ ▸ ✓  2 closed steps"));
    assert!(
        closed.is_some_and(|r| r.ends_with(" ✓ 1 · ✗ 1")),
        "{rows:?}"
    );
    assert!(!rows.join("\n").contains("parent"), "{rows:?}");
    watch.press("\x1b[B\x1b[B\x1b[B\x1b[C");
    watch.until("parent");
    let rows = watch.rows();
    let at = rows.iter().position(|r| r.contains("parent")).unwrap();
    assert!(rows[at].contains("━━━━━━ 1/1 · "), "{rows:?}");
    assert!(!rows.join("\n").contains("child") && !rows.join("\n").contains("not needed"));
    watch.press("\x1b[B\x1b[B\x1b[C");
    watch.until("✓  1 closed step");
    watch.press("\x1b[B\x1b[C");
    watch.until("✓  child");
}

/// With no id, the watch boxes today under a line with 📅, the day and its counts, groups the trails by
/// project under a 📁 heading, its active trails first, and ends with `Show more`, which `enter` turns into every trail by day and `Show less`.
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
    assert!(
        rows[0].starts_with("  📅 ") && rows[0].ends_with(" · 1 active · 0 done"),
        "{rows:?}"
    );
    assert_eq!(rows[1..4], ["  ", "  📁 repo", "  "], "{rows:?}");
    assert_eq!(rows[4], "  ▾ ○  the trail", "{rows:?}");
    assert!(!rows.join("\n").contains("an old trail"), "{rows:?}");
    watch.press("\x1b[B\x1b[B");
    watch.until("▌ ▸ Show more");
    watch.press("\r");
    watch.until("▾ Show less");
    watch.until("an old trail");
}

/// The watch cuts a long step title and a long trail line's title with `…` to fit the width.
#[test]
fn watch_cuts_long_titles_to_the_width() {
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
    assert!(rows.iter().all(|r| r.chars().count() <= 40), "{rows:?}");
    for title in ["done trail one", "open step one"] {
        let row = rows.iter().find(|r| r.contains(title));
        assert!(row.is_some_and(|r| r.contains('…')), "{rows:?}");
    }
    assert!(!rows.join("\n").contains("twelve"), "{rows:?}");
}

/// A trail's line prints its steps and worked time dim at the right of its title, and no count of its
/// rules.
#[test]
fn watch_prints_a_trail_lines_details_under_its_title() {
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
    let row = |what: &str| rows.iter().find(|l| l.contains(what)).unwrap();
    let first = row("first");
    assert!(
        first.starts_with("  ▸ ✓  first ") && first.ends_with(" 1 step · 5s"),
        "{rows:?}"
    );
    let second = row("second");
    assert!(
        second.starts_with("  ▸ ✓  second ") && second.ends_with(" 12 steps · 2m 0s"),
        "{rows:?}"
    );
    assert!(!rows.join("\n").contains("rule"), "{rows:?}");
    let colour = watch.coloured(Duration::ZERO);
    assert!(colour.contains("\x1b[2m1 step · 5s"), "{colour:?}");
}

/// A closed trail's line whose steps were not all done counts them by mark at its right.
#[test]
fn watch_counts_a_closed_trails_steps_by_mark() {
    let repo = Repo::new();
    let by = support::account();
    let at = support::now() - 4_000;
    let trail = support::step_note(&repo, &by, None, "the trail", at);
    let part = support::step_note(&repo, &by, Some(&trail), "done part", at);
    support::done_note(&repo, &by, &part, at + 5);
    let gone = add(&repo, &["--under", &trail.id[..8], "--", "gone part"]);
    let o = cli(&repo, &["step", "drop", &gone[..8], "--", "not needed"]);
    assert!(o.status.success(), "{}", err(&o));
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.size(24, 100);
    watch.until("the trail");
    let rows = watch.rows();
    let row = rows.iter().find(|r| r.contains("the trail")).unwrap();
    assert!(row.ends_with(" ✓ 1 · ✗ 1 · 5s"), "{rows:?}");
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
    watch.size(32, 80);
    watch.until("▸ Show more");
    watch.press("a");
    watch.until("an old trail");
    watch.press("\x1b[B\x1b[B");
    watch.until("▌ │ ▸ ✓  an old trail");
    watch.press("a");
    watch.until("▌ │ ▾ ○  the trail");
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

/// A changed done step prints `˜` after its mark and `changed` at its right, and no step prints its
/// place.
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
    watch.until("✓˜  the part");
    let rows = watch.rows();
    let row = rows.iter().find(|r| r.contains("the part")).unwrap();
    assert!(row.contains(" changed · "), "{rows:?}");
    assert!(!rows.join("\n").contains("shelf/a.py"), "{rows:?}");
}

/// A revised done step prints `↻ revised by` the step that revised it at its right, and its working trail
/// keeps the spinner with `↻` after it.
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
    watch.size(24, 100);
    watch.until("1 closed step");
    watch.press("\x1b[B\x1b[B\x1b[B\x1b[C");
    watch.until("✓˜  the part");
    let rows = watch.rows();
    let row = rows.iter().find(|r| r.contains("the part")).unwrap();
    assert!(
        row.contains(&format!("↻ revised by {} · ", &next[..8])),
        "{rows:?}"
    );
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
        .position(|r| r.contains("a trail whose"))
        .unwrap_or_else(|| panic!("{rows:?}"));
    assert!(
        rows[at + 1].ends_with("well past forty characters"),
        "{rows:?}"
    );
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

/// On a terminal with room, the tree prints under a box with the dude and what celis says of its trail:
/// how it stands, then the rule it learned, typed in.
#[test]
fn watch_says_what_celis_brought_the_trail_beside_the_dude() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.size(24, 80);
    watch.until("the trail");
    let screen = watch.screen_rows();
    assert!(
        screen[1].starts_with("  │ ▄    ▙▖  ├─ 📈 This trail so far ─"),
        "{screen:?}"
    );
    assert!(screen[3].contains("│ 0 of 0 steps done. "), "{screen:?}");
    assert!(watch.rows()[0].ends_with(" · 1 active · 0 done"));
    let scope = format!("trail:{}", &trail[..8]);
    let rule = "When you add a page: copy the frame, because it holds the header.";
    let o = cli(&repo, &["decide", "--scope", &scope, "--", rule]);
    assert!(o.status.success(), "{}", err(&o));
    watch.until("💡 Learned");
    watch.until("│ When you add a page: copy the frame.");
}

/// When a step starts with a rule live for it while the watch runs, celis says the rule paid off: the
/// step got it to keep; before, it says only how the trail stands, with no count of zero.
#[test]
fn watch_says_a_rule_paid_off_when_a_step_starts_with_one() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let scope = format!("trail:{}", &trail[..8]);
    let rule = "When you ship a page: run the checks.";
    let o = cli(&repo, &["decide", "--scope", &scope, "--", rule]);
    assert!(o.status.success(), "{}", err(&o));
    let step = add(&repo, &["--under", &trail[..8], "--", "the page"]);
    std::thread::sleep(Duration::from_millis(1100));
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.size(24, 80);
    watch.until("📈 This trail so far");
    watch.until("│ I learned 1 rule here. ");
    std::thread::sleep(Duration::from_millis(1100));
    let o = cli(&repo, &["step", "start", &step[..8]]);
    assert!(o.status.success(), "{}", err(&o));
    watch.until("🎉 Rule paid off");
    watch.until("│ Starting “the page”, your agent got 1 rule to keep, like: when");
}

/// What celis says is the newest value of every trail the watch shows: a rule learned in an older trail
/// reaches the card at once.
#[test]
fn watch_says_the_newest_value_of_every_trail() {
    let repo = Repo::new();
    let older = add(&repo, &["--", "the older trail"]);
    add(&repo, &["--new", "--", "the newer trail"]);
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.size(24, 80);
    watch.until("the older trail");
    let scope = format!("trail:{}", &older[..8]);
    let rule = "When you ship a page: run the checks.";
    let o = cli(&repo, &["decide", "--scope", &scope, "--", rule]);
    assert!(o.status.success(), "{}", err(&o));
    watch.until("│ When you ship a page: run the checks.");
}

/// A scene that fits only once the window widens shows still at once: the walker already cheering at
/// the summit, no climb.
#[test]
fn watch_shows_a_scene_still_when_the_window_widens() {
    let repo = Repo::new();
    add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &["tree"]);
    watch.size(40, 80);
    watch.until("the trail");
    watch.size(40, 140);
    watch.until("\\^o^/ |>");
    assert!(!watch.shown(Duration::ZERO).contains("•_•"));
}
