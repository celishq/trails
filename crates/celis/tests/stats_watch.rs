//! CIP-02, "Stats".

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

/// On a terminal, `stats` draws its cards: the effect, reuse, rules, work, time, outcome and projects,
/// a number from fewer than 10 records says so, and the keys it reads end the screen
/// (`spec/02-cli.md`, "Stats").
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
        "no data before",
        "01 REUSE",
        "02 RULES USED",
        "03 WORK · 14 DAYS",
        "04 TIME A STEP TOOK",
        "05 OUTCOME",
        "✓ done",
        "06 PROJECTS · TIME",
        "repo",
    ] {
        assert!(shown.contains(text), "{text:?} in\n{shown}");
    }
    watch.until("←→↑↓ move · enter open · m copy markdown · q quit");
}

/// With no store, `stats` on a terminal draws the empty board and writes nothing, ignores `enter`, then
/// draws the cards when the first step closes (`spec/02-cli.md`, "Stats").
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

/// With an open trail and no closed step, `stats` on a terminal draws the empty board, and `q` ends it
/// (`spec/02-cli.md`, "Stats").
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
/// Markdown table, and `←` goes back to the cards (`spec/02-cli.md`, "Stats").
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

/// On a terminal, `stats <id>` draws the step with the steps it follows and the steps that follow it,
/// and `enter` goes to the selected one (`spec/02-cli.md`, "Stats").
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

/// Below 60 columns, `stats` prints one card a row and its numbers as plain text
/// (`spec/02-cli.md`, "Stats").
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
/// steps were followed and rules used (`spec/02-cli.md`, "Stats").
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
        "no rule used yet",
        "—",
    ] {
        assert!(!shown.contains(empty), "{empty:?} in\n{shown}");
    }
}

/// Off a terminal, `stats --demo` prints the made-up trails' follows, uses and total from a folder it
/// removes, writes nothing to the store, and refuses an id (`spec/02-cli.md`, "Stats").
#[test]
fn stats_demo_prints_and_writes_nothing() {
    let repo = Repo::new();
    let temp = tempfile::tempdir().unwrap();
    let env = [("TMPDIR", temp.path().to_str().unwrap())];
    let o = repo.run(&repo.path(), &["stats", "--demo"], &env, "");
    assert!(o.status.success(), "{}", err(&o));
    let text = out(&o);
    for part in [" · followed ", "decision ", "total "] {
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
