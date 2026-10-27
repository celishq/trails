//! CIP-06, "Reuse".

use crate::support;

use celis_nostr::Note;
use serde_json::Value;
use support::{
    account, add, added, cli, err, finish, links, now, objects, out, put, refusal, step, step_note,
    step_tags, Repo,
};

/// The id `pattern -- <text>` prints.
fn decide(repo: &Repo, text: &str) -> String {
    let o = cli(repo, &["pattern", "--", text]);
    assert!(o.status.success(), "{}", err(&o));
    out(&o).trim().to_string()
}

/// `step add --uses` takes a live pattern and writes its whole id, and refuses a step of another trail
/// as a usage error (`spec/06-reuse.md`, "Uses").
#[test]
fn uses_takes_a_pattern_and_refuses_a_step() {
    let repo = Repo::new();
    let pattern = decide(
        &repo,
        "When a test adds two steps: wait 1.1 s between them.",
    );
    let past = add(&repo, &["--", "the senior price"]);
    finish(&repo, &past[..8]);
    let trail = add(&repo, &["--", "the student price"]);
    let id = add(
        &repo,
        &[
            "--under",
            &trail[..8],
            "--uses",
            &pattern[..8],
            "--",
            "a student pays 70%",
        ],
    );
    let json = step(&repo, &trail, &id);
    assert_eq!(links(&json, "uses"), [pattern], "{json}");
    let o = cli(
        &repo,
        &[
            "step",
            "add",
            "--json",
            "--under",
            &trail[..8],
            "--uses",
            &past[..8],
            "--",
            "a member pays 90%",
        ],
    );
    refusal(&o, 2, "usage");
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

/// `step add --replaces` takes a done step of another trail: the new step goes where `step add` puts it,
/// the old one stays done, and it and its trail read as revised while they stay done (`spec/06-reuse.md`,
/// "Revisions").
#[test]
fn replaces_takes_a_done_step_and_reads_it_as_revised() {
    let repo = Repo::new();
    let old_trail = &added(&repo, "s1", &["--", "the badge"])[0];
    let part = add(&repo, &["--under", &old_trail[..8], "--", "the colour"]);
    finish(&repo, &part[..8]);
    finish(&repo, &old_trail[..8]);
    let trail = add(&repo, &["--", "fix the badge"]);
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

/// The line of `text` that holds `id`.
fn line<'a>(text: &'a str, id: &str) -> &'a str {
    text.lines().find(|l| l.contains(&id[..8])).unwrap_or("")
}

/// `stats` with no record says so, and prints nothing with `--json`; with a record and no pattern, it
/// prints only the total; it refuses an unknown id with `no-step` (`spec/02-cli.md`, "Stats").
#[test]
fn stats_says_when_nothing_was_reused() {
    let repo = Repo::new();
    assert_eq!(
        out(&cli(&repo, &["stats"])).trim(),
        "Nothing was reused yet."
    );
    assert_eq!(out(&cli(&repo, &["stats", "--json"])), "");
    add(&repo, &["--", "the trail"]);
    assert_eq!(
        out(&cli(&repo, &["stats"])).trim(),
        "total 0 patterns in the index · last 7 days: 0 written, 0 replaced, 0 gaps"
    );
    refusal(&cli(&repo, &["stats", "--json", "ffffffff"]), 1, "no-step");
}

/// A step and the pattern it uses, each done: gives `(step, pattern)`.
fn used(repo: &Repo) -> (String, String) {
    let pattern = decide(repo, "When a price rounds: round down to the cent.");
    let trail = add(repo, &["--new", "--", "the student price"]);
    let id = add(
        repo,
        &[
            "--under",
            &trail[..8],
            "--uses",
            &pattern[..8],
            "--",
            "a student pays 70%",
        ],
    );
    finish(repo, &id[..8]);
    (id, pattern)
}

/// `stats` prints each used pattern with its uses, then the total (`spec/02-cli.md`, "Stats").
#[test]
fn stats_prints_the_used_patterns_and_the_total() {
    let repo = Repo::new();
    let (_, pattern) = used(&repo);
    let text = out(&cli(&repo, &["stats"]));
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines[0].starts_with(&format!("{} uses:1 ", &pattern[..8])),
        "{text}"
    );
    assert!(
        lines[1].starts_with("total 1 pattern in the index"),
        "{text}"
    );
    let json = objects(&repo, &["stats"]);
    assert_eq!(json[0]["pattern"]["uses"], 1, "{json:?}");
    assert_eq!(json[1]["total"]["index"], 1, "{json:?}");
}

/// `stats <id>` prints the step with its grade, and nothing more.
#[test]
fn stats_of_a_step_prints_the_step() {
    let repo = Repo::new();
    let (id, _) = used(&repo);
    let text = out(&cli(&repo, &["stats", &id[..8]]));
    assert_eq!(text.lines().count(), 1, "{text}");
    assert!(line(&text, &id).starts_with(&id[..8]), "{text}");
}

/// A reader ignores a `uses` that names a step, which older writers wrote: `stats` counts only the
/// pattern, and `tree` shows no link to the step (`spec/06-reuse.md`, "Uses").
#[test]
fn a_uses_that_names_a_step_is_ignored() {
    let repo = Repo::new();
    let pattern = decide(&repo, "When a price rounds: round down to the cent.");
    let past = add(&repo, &["--", "the senior price"]);
    finish(&repo, &past[..8]);
    let trail = add(&repo, &["--", "the student price"]);
    let mut tags = step_tags((&repo.project_id(), "repo"));
    tags.push(vec!["under".into(), trail.clone()]);
    tags.push(vec!["uses".into(), past.clone()]);
    tags.push(vec!["uses".into(), pattern.clone()]);
    let old = Note::write(&account(), 1111u16, now(), tags, "a student pays 70%").unwrap();
    put(&repo, &old);
    let json = objects(&repo, &["stats"]);
    assert_eq!(json[0]["pattern"]["uses"], 1, "{json:?}");
    assert!(!format!("{json:?}").contains(&past[..8]), "{json:?}");
    let tree = out(&cli(&repo, &["tree", &trail[..8], "--full"]));
    assert!(tree.contains("a student pays 70%"), "{tree}");
    assert!(!tree.contains(&past[..8]), "{tree}");
}

/// A new ask that `--replaces` a done ask goes on top and carries the link: the done ask reads as revised
/// once (`spec/03-steps.md`, "Dropping"; `spec/06-reuse.md`, "Revisions").
#[test]
fn a_new_ask_that_replaces_a_done_ask_links_it_once() {
    let repo = Repo::new();
    let old = add(&repo, &["--", "the badge"]);
    finish(&repo, &old[..8]);
    let fix = &added(
        &repo,
        "s2",
        &["--replaces", &old[..8], "--", "fix the badge"],
    )[0];
    let json = step(&repo, fix, fix);
    assert!(links(&json, "under").is_empty(), "{json}");
    assert_eq!(links(&json, "replaces"), [old.as_str()], "{json}");
    let revised = step(&repo, &old, &old);
    assert_eq!(revised["revised_by"], serde_json::json!([fix]), "{revised}");
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

/// The ids of an older trail of `text`, which holds a session's first ask of the same text, and of that
/// ask.
fn older(repo: &Repo, text: &str) -> (String, String) {
    let trail = step_note(repo, &account(), None, text, now() - 60);
    let ask = step_note(repo, &account(), Some(&trail), text, now() - 59);
    (trail.id, ask.id)
}

/// An older trail's line shows its own drop, not its first ask's done (`spec/04-trails.md`, "Trails").
#[test]
fn a_dropped_trail_shows_its_own_drop() {
    let repo = Repo::new();
    let (trail, ask) = older(&repo, "the trail");
    finish(&repo, &ask[..8]);
    let more = add(&repo, &["--under", &trail[..8], "--", "more"]);
    let o = cli(&repo, &["step", "drop", &trail[..8], "--", "abandoned"]);
    assert!(o.status.success(), "{}", err(&o));
    let head = step(&repo, &trail, &trail);
    assert_eq!(head["reason"], "abandoned", "{head}");
    assert_eq!(head["proof"], Value::Null, "{head}");
    assert!(!more.is_empty());
}
