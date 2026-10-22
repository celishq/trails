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

/// On a terminal, `stats` draws its board: the four headline tiles, then each section closed to its
/// card, and the key line under them.
#[test]
fn stats_on_a_terminal_draws_its_board() {
    let (repo, _) = worked();
    let mut watch = Watch::start(&repo, &["stats"]);
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

/// `enter` opens the picked section to its details and names close in the key line, `a` opens every
/// section, and ←→ picks a day once Output and speed is open.
#[test]
fn stats_opens_a_section_and_all() {
    let (repo, _) = worked();
    let mut watch = Watch::start(&repo, &["stats"]);
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

/// On a screen shorter than an opened section, ↓ scrolls through it before it picks the next section.
#[test]
fn stats_scrolls_through_a_tall_section() {
    let (repo, _) = worked();
    let mut watch = Watch::start(&repo, &["stats"]);
    watch.size(10, 110);
    watch.until("resolved steps");
    watch.press("\r");
    watch.until("▾ Learning");
    watch.until("enter close");
    watch.without("weekly rule usage");
    watch.press("\x1b[B");
    watch.until("weekly rule usage");
    watch.until("enter close");
    watch.press("\x1b[B\x1b[B\x1b[B");
    watch.until("enter open");
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

/// On a narrow screen `stats` still draws every section, the tiles one a row.
#[test]
fn stats_on_a_narrow_screen_draws_every_section() {
    let (repo, _) = worked();
    let mut watch = Watch::start(&repo, &["stats"]);
    watch.size(80, 40);
    watch.until("▸ RESOURCES");
    let rows = watch.screen_rows();
    let at = |text: &str| rows.iter().position(|r| r.contains(text));
    assert!(at("REUSE RATE") < at("FIRST-PASS"), "{rows:#?}");
    assert!(at("▸ LEARNING").is_some(), "{rows:#?}");
}

/// `stats --demo` fills the board from a made-up month: reuse grows and every measure has the prior
/// 14 days to compare with.
#[test]
fn stats_demo_fills_the_board() {
    let repo = Repo::new();
    let mut watch = Watch::start(&repo, &["stats", "--demo"]);
    watch.size(40, 110);
    watch.until("20 of 42 successful");
    let shown = watch.screen_rows().join("\n");
    for text in ["40 of 44 resolved", "5% → 57% by week", "● 2 active"] {
        assert!(shown.contains(text), "{text:?} in\n{shown}");
    }
    assert!(!shown.contains("no prior steps"), "{shown}");
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
    for part in [" · followed ", "rule ", "total "] {
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
