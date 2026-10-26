//! CIP-06, "Reuse".

mod support;

use serde_json::Value;
use support::{add, added, cli, err, finish, links, objects, out, refusal, step, Repo};

/// The id `pattern -- <text>` prints.
fn decide(repo: &Repo, text: &str) -> String {
    let o = cli(repo, &["pattern", "--", text]);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

/// `step add --uses` takes a live pattern and a step of another trail, writes each whole id, and gives
/// a trail id as its first ask (`spec/06-reuse.md`, "Uses").
#[test]
fn uses_takes_a_pattern_and_a_step_of_another_trail() {
    let repo = Repo::new();
    let pattern = decide(
        &repo,
        "When a test adds two steps: wait 1.1 s between them.",
    );
    let past = added(&repo, "s1", &["--", "the senior price"]);
    let (past_ask, past_trail) = (&past[0], &past[1]);
    finish(&repo, &past_ask[..8]);
    let trail = add(&repo, &["--new", "--", "the student price"]);
    let id = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--uses",
            &pattern[..8],
            "--uses",
            &past_trail[..8],
            "--",
            "a student pays 70%",
        ],
    );
    let json = step(&repo, &trail, &id);
    assert_eq!(links(&json, "uses"), [pattern, past_ask.clone()], "{json}");
    assert_eq!(links(&json, "under"), [trail], "{json}");
}

/// `step add --uses` refuses an id no pattern or step has with `no-step`, and a step of its own trail
/// as a usage error (`spec/06-reuse.md`, "Uses").
#[test]
fn uses_refuses_an_unknown_id_and_a_step_of_the_same_trail() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "part"]);
    let o = cli(
        &repo,
        &[
            "step", "add", "--json", "--uses", "ffffffff", "--", "unknown",
        ],
    );
    refusal(&o, 1, "no-step");
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--json",
            "--under",
            &trail[..8],
            "--uses",
            &part[..8],
            "--",
            "same trail",
        ],
    );
    refusal(&o, 2, "usage");
}

/// On a session's new trail and its ask, the `uses` tags go on the ask only
/// (`spec/06-reuse.md`, "Uses").
#[test]
fn uses_goes_on_the_ask_of_a_new_trail() {
    let repo = Repo::new();
    let pattern = decide(&repo, "When a price rounds: round down to the cent.");
    let written = added(&repo, "s1", &["--uses", &pattern[..8], "--", "the price"]);
    let (ask, trail) = (&written[0], &written[1]);
    assert_eq!(links(&step(&repo, trail, ask), "uses"), [pattern]);
    assert!(links(&step(&repo, trail, trail), "uses").is_empty());
}

/// `step add --replaces` takes a done step of another trail: the new step goes where `step add` puts it,
/// the old one stays done, and it and its trail read as revised while they stay done (`spec/06-reuse.md`,
/// "Revisions").
#[test]
fn replaces_takes_a_done_step_and_reads_it_as_revised() {
    let repo = Repo::new();
    let old = added(&repo, "s1", &["--", "the badge"]);
    let (old_ask, old_trail) = (&old[0], &old[1]);
    let part = add(&repo, &["--under", &old_ask[..8], "--", "the colour"]);
    finish(&repo, &part[..8]);
    finish(&repo, &old_ask[..8]);
    let trail = add(&repo, &["--new", "--", "fix the badge"]);
    let fix = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--replaces",
            &part[..8],
            "--",
            "the badge keeps its colour",
        ],
    );
    let json = step(&repo, &trail, &fix);
    assert_eq!(links(&json, "under"), [trail], "{json}");
    assert_eq!(links(&json, "replaces"), [part.as_str()], "{json}");
    let revised = step(&repo, old_trail, &part);
    assert_eq!(revised["revised_by"], serde_json::json!([fix]), "{revised}");
    let head = objects(&repo, &["tree", &old_trail[..8]]);
    assert_eq!(head[0]["trail"]["state"], "done", "{head:?}");
    assert_eq!(
        head[0]["trail"]["revised"],
        serde_json::json!([part]),
        "{head:?}"
    );
    let tree = out(&cli(&repo, &["tree", &old_trail[..8], "--full"]));
    let line = format!(" · revised by {}", &fix[..8]);
    assert!(tree.contains(&line), "{tree}");
}

/// `step add --replaces` takes a session's trail as its first ask, which reads as revised; a trail with no
/// first ask stays refused (`spec/02-cli.md`, "Step").
#[test]
fn replaces_takes_a_trail_as_its_first_ask() {
    let repo = Repo::new();
    let old = added(&repo, "s1", &["--", "the badge"]);
    let (old_ask, old_trail) = (&old[0], &old[1]);
    finish(&repo, &old_ask[..8]);
    let fix = added(
        &repo,
        "s1",
        &["--replaces", &old_trail[..8], "--", "the badge is green"],
    );
    let json = step(&repo, old_trail, &fix[0]);
    assert_eq!(links(&json, "replaces"), [old_ask.as_str()], "{json}");
    let lone = add(&repo, &["--", "a lone trail"]);
    let o = cli(&repo, &["step", "add", "--replaces", &lone[..8], "--", "x"]);
    assert_eq!(o.status.code(), Some(2), "{}", err(&o));
}

/// `step add --replaces` still refuses a dropped step with `closed` (`spec/03-steps.md`,
/// "Dropping").
#[test]
fn replaces_refuses_a_dropped_step() {
    let repo = Repo::new();
    let trail = add(&repo, &["--", "the trail"]);
    let part = add(&repo, &["--under", &trail[..8], "--", "part"]);
    let o = cli(&repo, &["step", "drop", &part[..8], "--", "not wanted"]);
    assert!(o.status.success(), "{}", err(&o));
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--json",
            "--replaces",
            &part[..8],
            "--",
            "again",
        ],
    );
    refusal(&o, 1, "closed");
}

/// A chain of follows: the ask `a` of one trail, `b` of a second that uses `a` and the pattern, and
/// `c` of a third that uses `b` and the pattern; each done. Gives `(a, b, c, pattern)`.
fn chain(repo: &Repo) -> (String, String, String, String) {
    let pattern = decide(repo, "When a price rounds: round down to the cent.");
    let a = added(repo, "s1", &["--", "a senior pays 80%"]).remove(0);
    finish(repo, &a[..8]);
    let follow = |source: &str, text: &str| {
        let trail = add(repo, &["--new", "--", text]);
        let step = add(
            repo,
            &[
                "--under",
                &trail[..8],
                "--uses",
                &source[..8],
                "--uses",
                &pattern[..8],
                "--",
                text,
            ],
        );
        finish(repo, &step[..8]);
        step
    };
    let b = follow(&a, "a student pays 70%");
    let c = follow(&b, "a member pays 90%");
    (a, b, c, pattern)
}

/// The line of `text` that holds `id`.
fn line<'a>(text: &'a str, id: &str) -> &'a str {
    text.lines().find(|l| l.contains(&id[..8])).unwrap_or("")
}

/// `stats` with nothing reused says so, and prints nothing with `--json`; it refuses an unknown id
/// with `no-step`.
#[test]
fn stats_says_when_nothing_was_reused() {
    let repo = Repo::new();
    add(&repo, &["--", "the trail"]);
    assert_eq!(
        out(&cli(&repo, &["stats"])).trim(),
        "Nothing was reused yet."
    );
    assert_eq!(out(&cli(&repo, &["stats", "--json"])), "");
    refusal(&cli(&repo, &["stats", "--json", "ffffffff"]), 1, "no-step");
}

/// `stats` prints each followed step, most followed at any depth first, then each used pattern, then
/// the total.
#[test]
fn stats_prints_the_followed_steps_the_used_patterns_and_the_total() {
    let repo = Repo::new();
    let (a, b, _, pattern) = chain(&repo);
    let text = out(&cli(&repo, &["stats"]));
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].starts_with(&a[..8]), "{text}");
    assert!(!text.contains("saved"), "{text}");
    assert!(lines[0].contains(" · followed 2"), "{text}");
    assert!(lines[1].starts_with(&b[..8]), "{text}");
    assert!(lines[1].contains(" · followed 1"), "{text}");
    assert!(
        lines[2].starts_with(&format!("pattern {}", &pattern[..8])),
        "{text}"
    );
    assert!(lines[2].ends_with(" · used 2"), "{text}");
    assert_eq!(lines[3], "total 2 follows · 2 uses", "{text}");
    let json = objects(&repo, &["stats"]);
    assert_eq!(json[0]["followed"]["id"], a, "{json:?}");
    assert_eq!(json[0]["followed"]["followed"], 2, "{json:?}");
    assert_eq!(json[2]["used"]["used"], 2, "{json:?}");
    assert_eq!(json[3]["total"]["follows"], 2, "{json:?}");
    assert!(json[3]["total"].get("saved").is_none(), "{json:?}");
}

/// `stats <id>` prints the step, then each step it follows, then each step that follows it, two spaces
/// further in a level.
#[test]
fn stats_of_a_step_prints_its_sources_then_its_followers() {
    let repo = Repo::new();
    let (a, b, c, _) = chain(&repo);
    let text = out(&cli(&repo, &["stats", &c[..8]]));
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].starts_with(&c[..8]), "{text}");
    assert!(
        lines[1].starts_with(&format!("  follows {}", &b[..8])),
        "{text}"
    );
    assert!(
        lines[2].starts_with(&format!("    follows {}", &a[..8])),
        "{text}"
    );
    let text = out(&cli(&repo, &["stats", &a[..8]]));
    assert!(line(&text, &b).starts_with("  followed "), "{text}");
    assert!(!text.contains("saved"), "{text}");
    assert!(line(&text, &c).starts_with("    followed "), "{text}");
}

/// `tree --full` prints a `follows` line under a step for each step it follows, and a `followed` line
/// when steps follow it; `tree --json` gives the count.
#[test]
fn tree_prints_follows_and_followed_lines() {
    let repo = Repo::new();
    let (a, b, _, _) = chain(&repo);
    let tree = out(&cli(&repo, &["tree", &b[..8], "--full"]));
    let follows = format!("follows  {} a senior pays 80%", &a[..8]);
    assert!(
        tree.lines()
            .any(|l| l.trim_start_matches([' ', '│']) == follows),
        "{tree}"
    );
    assert!(
        tree.lines()
            .any(|l| l.trim_start_matches([' ', '│']) == "followed 1"),
        "{tree}"
    );
    let json = objects(&repo, &["tree", &b[..8]]);
    let at = json
        .iter()
        .find_map(|v| v.get("step").or(v.get("trail")))
        .unwrap();
    assert_eq!(at["followed"], 1, "{json:?}");
}

/// A session's first ask that `--replaces` a done step carries the link, and its trail does not: the done
/// step reads as revised once, and no `joined` line prints (`spec/06-reuse.md`, "Revisions").
#[test]
fn a_new_trail_that_replaces_a_done_step_links_it_once() {
    let repo = Repo::new();
    let old = added(&repo, "s1", &["--", "the badge"]);
    finish(&repo, &old[0][..8]);
    let o = repo.run(
        &repo.path(),
        &[
            "step",
            "add",
            "--replaces",
            &old[0][..8],
            "--",
            "fix the badge",
        ],
        &[("CLAUDE_CODE_SESSION_ID", "s2")],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    let fix: Vec<String> = out(&o).lines().map(str::to_string).collect();
    let revised = step(&repo, &old[1], &old[0]);
    assert_eq!(
        revised["revised_by"],
        serde_json::json!([fix[0]]),
        "{revised}"
    );
    let trail = step(&repo, &fix[1], &fix[1]);
    assert!(links(&trail, "replaces").is_empty(), "{trail}");
    let again = added(&repo, "s2", &["--", "the next fix"]);
    finish(&repo, &again[0][..8]);
    let o = repo.run(
        &repo.path(),
        &[
            "step",
            "add",
            "--replaces",
            &again[0][..8],
            "--",
            "fix it again",
        ],
        &[("CLAUDE_CODE_SESSION_ID", "s2")],
        "",
    );
    assert!(o.status.success(), "{}", err(&o));
    assert!(!err(&o).contains("joined"), "{}", err(&o));
}

/// `step add` with `--uses` or a done `--replaces` refuses, as a usage error, an open step of the same
/// place and words, and writes nothing (`spec/02-cli.md`, "Step").
#[test]
fn a_link_on_a_step_that_exists_is_refused() {
    let repo = Repo::new();
    let pattern = decide(&repo, "When a price rounds: round down to the cent.");
    let trail = add(&repo, &["--", "the trail"]);
    add(&repo, &["--under", &trail[..8], "--", "the badge"]);
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--json",
            "--under",
            &trail[..8],
            "--uses",
            &pattern[..8],
            "--",
            "the badge",
        ],
    );
    refusal(&o, 2, "usage");
}

/// `--uses` writes each id once (`spec/06-reuse.md`, "Uses").
#[test]
fn uses_writes_each_id_once() {
    let repo = Repo::new();
    let pattern = decide(&repo, "When a price rounds: round down to the cent.");
    let trail = add(&repo, &["--", "the trail"]);
    let id = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--uses",
            &pattern[..8],
            "--uses",
            &pattern,
            "--",
            "a step",
        ],
    );
    assert_eq!(links(&step(&repo, &trail, &id), "uses"), [pattern]);
}

/// `--uses` of a trail with no first ask of its text names the trail, and `stats` and `tree` count the
/// follow on it (`spec/06-reuse.md`, "Uses").
#[test]
fn uses_of_a_trail_with_no_twin_names_the_trail() {
    let repo = Repo::new();
    let past = add(&repo, &["--", "the senior price"]);
    let part = add(&repo, &["--under", &past[..8], "--", "a part"]);
    finish(&repo, &part[..8]);
    let trail = add(&repo, &["--new", "--", "the student price"]);
    let id = add(
        &repo,
        &["--under", &trail[..8], "--uses", &past[..8], "--", "a step"],
    );
    assert_eq!(links(&step(&repo, &trail, &id), "uses"), [past.as_str()]);
    let text = out(&cli(&repo, &["stats", &past[..8]]));
    assert!(line(&text, &id).starts_with("  followed "), "{text}");
    assert_eq!(step(&repo, &past, &past)["followed"], 1);
}

/// A session trail's line counts `followed` on its first ask in text and in `--json` alike.
#[test]
fn a_session_trail_counts_followed_on_its_first_ask() {
    let repo = Repo::new();
    let past = added(&repo, "s1", &["--", "the senior price"]);
    finish(&repo, &past[0][..8]);
    let trail = add(&repo, &["--new", "--", "the student price"]);
    add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--uses",
            &past[1][..8],
            "--",
            "a step",
        ],
    );
    assert_eq!(step(&repo, &past[1], &past[1])["followed"], 1);
}

/// A dropped trail's line shows its own drop, not its first ask's done.
#[test]
fn a_dropped_trail_shows_its_own_drop() {
    let repo = Repo::new();
    let written = added(&repo, "s1", &["--", "the trail"]);
    finish(&repo, &written[0][..8]);
    let more = add(&repo, &["--under", &written[1][..8], "--", "more"]);
    let o = cli(
        &repo,
        &["step", "drop", &written[1][..8], "--", "abandoned"],
    );
    assert!(o.status.success(), "{}", err(&o));
    let head = step(&repo, &written[1], &written[1]);
    assert_eq!(head["reason"], "abandoned", "{head}");
    assert_eq!(head["proof"], Value::Null, "{head}");
    assert!(!more.is_empty());
}
