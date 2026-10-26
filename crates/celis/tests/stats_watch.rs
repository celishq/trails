//! The view `stats` shows on a terminal.

mod support;

use support::watch::Watch;
use support::{add, cli, err, out, refusal, Repo};

/// Runs `celis <args>` in the main checkout, and asserts it succeeds.
fn ok(repo: &Repo, args: &[&str]) {
    let o = cli(repo, args);
    assert!(o.status.success(), "{}", err(&o));
}

/// A repo with a trail and one step under it done as planned; the step's id.
fn worked() -> (Repo, String) {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "the part"]);
    ok(
        &repo,
        &["step", "done", &part[..8], "--as-planned", "--", "it works"],
    );
    (repo, part)
}

/// A `pbcopy` in the repo's home that writes what it copies to `$CELIS_HOME/copied`.
fn clipboard(repo: &Repo) {
    let bin = repo.home().join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let copy = bin.join("pbcopy");
    std::fs::write(&copy, "#!/bin/sh\ncat > \"$CELIS_HOME/copied\"\n").unwrap();
    std::fs::set_permissions(&copy, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
}

/// What the repo's `pbcopy` copied last.
fn copied(repo: &Repo) -> String {
    std::fs::read_to_string(repo.home().join("copied")).unwrap()
}

/// On a terminal, `stats` draws its cards: the effect, reuse, patterns, work, time, outcome and projects,
/// a number from fewer than 10 records says so, and the keys it reads end the screen.
#[test]
fn stats_on_a_terminal_draws_its_cards() {
    let (repo, _) = worked();
    let mut watch = Watch::start(&repo, &["stats"]);
    watch.size(40, 100);
    watch.until("06 PROJECTS");
    let shown = watch.rows().join("\n");
    for text in [
        "CELIS STATS",
        "00 EFFECT",
        "AS PLANNED",
        "01 REUSE",
        "02 PATTERNS USED",
        "03 WORK · 14 DAYS",
        "04 TIME A STEP TOOK",
        "05 OUTCOME",
        "✓ done",
        "06 PROJECTS · TIME",
        "repo",
    ] {
        assert!(shown.contains(text), "{text:?} in\n{shown}");
    }
    assert!(!shown.contains("no data before"), "{shown}");
    watch.until("←→↑↓ move · enter open · m copy markdown · q quit");
}

/// With no store, `stats` on a terminal draws the empty board and writes nothing, ignores `enter`, then
/// draws the cards when the first step closes.
#[test]
fn stats_with_no_store_draws_the_empty_board_until_a_step_closes() {
    let repo = Repo::new();
    let mut watch = Watch::start(&repo, &["stats"]);
    watch.size(40, 100);
    watch.until("NO STATS YET");
    let shown = watch.rows().join("\n");
    for text in ["CELIS STATS", "first step", "celis stats --demo"] {
        assert!(shown.contains(text), "{text:?} in\n{shown}");
    }
    assert!(!shown.contains("00 EFFECT"), "{shown}");
    watch.until("q quit");
    assert!(!repo.store().join("shared").exists());
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "the part"]);
    watch.until("NO STATS YET");
    watch.press("\r");
    ok(
        &repo,
        &["step", "done", &part[..8], "--as-planned", "--", "it works"],
    );
    watch.until("00 EFFECT");
    let shown = watch.rows().join("\n");
    assert!(!shown.contains("CELIS STATS ›"), "{shown}");
}

/// With an open trail and no closed step, `stats` on a terminal draws the empty board, and `q`
/// ends it.
#[test]
fn stats_with_no_closed_step_draws_the_empty_board() {
    let repo = Repo::new();
    add(&repo, &["--", "the trail"]);
    let mut watch = Watch::start(&repo, &["stats"]);
    watch.size(40, 100);
    watch.until("NO STATS YET");
    watch.press("q");
    assert!(watch.child.wait().unwrap().success());
}

/// `enter` opens the picked card to a row a record, `c` copies the selected row's id, `m` the card as a
/// Markdown table, and `←` goes back to the cards.
#[test]
fn stats_opens_a_card_and_copies_it() {
    let (repo, part) = worked();
    clipboard(&repo);
    let mut watch = Watch::start(&repo, &["stats"]);
    watch.size(40, 100);
    watch.until("06 PROJECTS");
    watch.press("\x1b[B\x1b[B\x1b[B\r");
    watch.until("CELIS STATS › 05 OUTCOME");
    watch.until(&format!("c copy {}", &part[..8]));
    assert!(watch.rows().join("\n").contains("the part"));
    watch.press("c");
    watch.until(&format!("copied {}", &part[..8]));
    assert_eq!(copied(&repo), &part[..8]);
    watch.press("m");
    watch.until("copied markdown");
    assert!(
        copied(&repo).starts_with("### 05 OUTCOME\n\n| State | Step |"),
        "{}",
        copied(&repo)
    );
    watch.press("\x1b[D");
    watch.until("06 PROJECTS");
}

/// An opened card taller than the screen scrolls with its selected row, and going back up to the first
/// row shows the card's title again.
#[test]
fn stats_scrolls_an_opened_card_back_to_its_title() {
    let (repo, _) = worked();
    let mut watch = Watch::start(&repo, &["stats"]);
    watch.size(10, 100);
    watch.until("00 EFFECT");
    watch.press("\x1b[B\x1b[B\r");
    watch.until("CELIS STATS › 03 WORK");
    watch.press(&"\x1b[B".repeat(13));
    watch.until("↑ ");
    watch.without("CELIS STATS › 03 WORK");
    watch.press(&"\x1b[A".repeat(13));
    watch.until("CELIS STATS › 03 WORK");
}

/// On a terminal, `stats <id>` draws the step with the steps it follows and the steps that follow it,
/// and `enter` goes to the selected one.
#[test]
fn stats_with_an_id_draws_its_route() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let source = add(&repo, &["--under", &trail[..8], "--", "the source"]);
    ok(
        &repo,
        &[
            "step",
            "done",
            &source[..8],
            "--as-planned",
            "--",
            "done once",
        ],
    );
    let next = add(&repo, &["--new", "--", "the next trail"]);
    let follower = add(
        &repo,
        &[
            "--under",
            &next[..8],
            "--uses",
            &source[..8],
            "--",
            "the follower",
        ],
    );
    ok(
        &repo,
        &[
            "step",
            "done",
            &follower[..8],
            "--as-planned",
            "--",
            "done again",
        ],
    );
    let mut watch = Watch::start(&repo, &["stats", &follower[..8]]);
    watch.size(40, 100);
    watch.until(&format!("CELIS STATS › {}", &follower[..8]));
    let shown = watch.rows().join("\n");
    for text in ["FOLLOWS", "the source", "the follower"] {
        assert!(shown.contains(text), "{text:?} in\n{shown}");
    }
    watch.until(&format!("c copy {}", &source[..8]));
    watch.press("\r");
    watch.until(&format!("CELIS STATS › {}", &source[..8]));
    watch.until("FOLLOWED BY");
}

/// Below 60 columns, `stats` prints one card a row and its numbers as plain text.
#[test]
fn stats_at_a_narrow_width_prints_plain_numbers() {
    let (repo, _) = worked();
    let mut watch = Watch::start(&repo, &["stats"]);
    watch.size(80, 50);
    watch.until("00 EFFECT");
    let rows = watch.rows();
    assert!(!rows.join("\n").contains("▀▀▀"), "{rows:#?}");
    assert!(
        rows.iter()
            .all(|r| !r.contains("01 REUSE") || !r.contains("02 RULES")),
        "{rows:#?}"
    );
}

/// `stats --demo` fills every card from made-up trails: each number of the effect shows its change, and
/// steps were followed and patterns used.
#[test]
fn stats_demo_fills_every_card() {
    let repo = Repo::new();
    let mut watch = Watch::start(&repo, &["stats", "--demo"]);
    watch.size(40, 100);
    watch.until("06 PROJECTS");
    let shown = watch.rows().join("\n");
    for empty in [
        "few records",
        "no step followed another yet",
        "no pattern used yet",
        "—",
    ] {
        assert!(!shown.contains(empty), "{empty:?} in\n{shown}");
    }
}

/// Off a terminal, `stats --demo` prints the made-up trails' follows, uses and total from a folder it
/// removes, writes nothing to the store, and refuses an id.
#[test]
fn stats_demo_prints_and_writes_nothing() {
    let repo = Repo::new();
    let temp = tempfile::tempdir().unwrap();
    let env = [("TMPDIR", temp.path().to_str().unwrap())];
    let o = repo.run(&repo.path(), &["stats", "--demo"], &env, "");
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    for part in [" · followed ", "pattern ", "total "] {
        assert!(text.contains(part), "{part:?}: {text}");
    }
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    assert!(!repo.store().join("shared").exists());
    let o = repo.run(
        &repo.path(),
        &["stats", "--demo", "abcd", "--json"],
        &env,
        "",
    );
    refusal(&o, 2, "usage");
}

/// With `--next`, On a terminal, `stats` draws its board: the four headline tiles, then each section closed to its
/// card, and the key line under them.
#[test]
fn next_stats_on_a_terminal_draws_its_board() {
    let (repo, _) = worked();
    let mut watch = Watch::start(&repo, &["stats", "--next"]);
    watch.size(40, 110);
    watch.until("▸ RESOURCES");
    let shown = watch.screen_rows().join("\n");
    for text in [
        "CELIS STATS",
        "Overview",
        "REUSE RATE",
        "FIRST-PASS SUCCESS",
        "RESOLVED STEPS",
        "MEDIAN STEP TIME",
        "▸ LEARNING",
        "▸ QUALITY",
        "▸ OUTPUT AND SPEED",
        "0 of 1 successful",
    ] {
        assert!(shown.contains(text), "{text:?} in\n{shown}");
    }
    watch.until("↑↓ section · enter open · a all · q quit");
}

/// With `--next`, With no store, `stats` on a terminal draws the empty board and writes nothing, ignores `enter`, then
/// draws the cards when the first step closes.
#[test]
fn next_stats_with_no_store_draws_the_empty_board_until_a_step_closes() {
    let repo = Repo::new();
    let mut watch = Watch::start(&repo, &["stats", "--next"]);
    watch.size(40, 100);
    watch.until("NO STATS YET");
    let shown = watch.rows().join("\n");
    for text in ["CELIS STATS", "first step", "celis stats --demo"] {
        assert!(shown.contains(text), "{text:?} in\n{shown}");
    }
    assert!(!shown.contains("REUSE RATE"), "{shown}");
    watch.until("q quit");
    assert!(!repo.store().join("shared").exists());
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "the part"]);
    watch.until("NO STATS YET");
    watch.press("\r");
    ok(
        &repo,
        &["step", "done", &part[..8], "--as-planned", "--", "it works"],
    );
    watch.until("REUSE RATE");
    let shown = watch.rows().join("\n");
    assert!(!shown.contains("NO STATS YET"), "{shown}");
}

/// With `--next`, `enter` opens the picked section to its details and names close in the key line, `a` opens every
/// section, and ←→ picks a day once Output and speed is open.
#[test]
fn next_stats_opens_a_section_and_all() {
    let (repo, _) = worked();
    let mut watch = Watch::start(&repo, &["stats", "--next"]);
    watch.size(60, 110);
    watch.until("▸ LEARNING");
    watch.press("\r");
    watch.until("▾ Learning");
    watch.until("REUSE · past steps reapplied");
    watch.until("enter close");
    watch.press("a");
    watch.until("▾ Output and speed");
    watch.until("←→ day");
}

/// With `--next`, On a screen shorter than an opened section, ↓ scrolls through it before it picks the next section.
#[test]
fn next_stats_scrolls_through_a_tall_section() {
    let (repo, _) = worked();
    let mut watch = Watch::start(&repo, &["stats", "--next"]);
    watch.size(10, 110);
    watch.until("resolved steps");
    watch.press("\r");
    watch.until("▾ Learning");
    watch.until("enter close");
    watch.without("weekly pattern usage");
    watch.press("\x1b[B");
    watch.until("weekly pattern usage");
    watch.until("enter close");
    watch.press("\x1b[B\x1b[B\x1b[B");
    watch.until("enter open");
}

/// With `--next`, On a terminal, `stats <id>` draws the step with the steps it follows and the steps that follow it,
/// and `enter` goes to the selected one.
#[test]
fn next_stats_with_an_id_draws_its_route() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let source = add(&repo, &["--under", &trail[..8], "--", "the source"]);
    ok(
        &repo,
        &[
            "step",
            "done",
            &source[..8],
            "--as-planned",
            "--",
            "done once",
        ],
    );
    let next = add(&repo, &["--new", "--", "the next trail"]);
    let follower = add(
        &repo,
        &[
            "--under",
            &next[..8],
            "--uses",
            &source[..8],
            "--",
            "the follower",
        ],
    );
    ok(
        &repo,
        &[
            "step",
            "done",
            &follower[..8],
            "--as-planned",
            "--",
            "done again",
        ],
    );
    let mut watch = Watch::start(&repo, &["stats", "--next", &follower[..8]]);
    watch.size(40, 100);
    watch.until(&format!("CELIS STATS › {}", &follower[..8]));
    let shown = watch.rows().join("\n");
    for text in ["FOLLOWS", "the source", "the follower"] {
        assert!(shown.contains(text), "{text:?} in\n{shown}");
    }
    watch.until(&format!("c copy {}", &source[..8]));
    watch.press("\r");
    watch.until(&format!("CELIS STATS › {}", &source[..8]));
    watch.until("FOLLOWED BY");
}

/// With `--next`, On a narrow screen `stats` still draws every section, the tiles one a row.
#[test]
fn next_stats_on_a_narrow_screen_draws_every_section() {
    let (repo, _) = worked();
    let mut watch = Watch::start(&repo, &["stats", "--next"]);
    watch.size(80, 40);
    watch.until("▸ RESOURCES");
    let rows = watch.screen_rows();
    let at = |text: &str| rows.iter().position(|r| r.contains(text));
    assert!(at("REUSE RATE") < at("FIRST-PASS"), "{rows:#?}");
    assert!(at("▸ LEARNING").is_some(), "{rows:#?}");
}

/// With `--next`, `stats --demo` fills the board from a made-up month: reuse grows and every measure has the prior
/// 14 days to compare with.
#[test]
fn next_stats_demo_fills_the_board() {
    let repo = Repo::new();
    let mut watch = Watch::start(&repo, &["stats", "--next", "--demo"]);
    watch.size(40, 110);
    watch.until("20 of 42 successful");
    let shown = watch.screen_rows().join("\n");
    for text in ["40 of 44 resolved", "5% → 57% by week", "● 2 active"] {
        assert!(shown.contains(text), "{text:?} in\n{shown}");
    }
    assert!(!shown.contains("no prior steps"), "{shown}");
}
